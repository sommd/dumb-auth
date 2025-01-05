use std::net::SocketAddr;

use clap::{ArgAction, Parser};
use tokio::net::TcpListener;

use dumb_auth::{AuthConfig, SessionExpiry};

#[derive(Debug, Parser)]
#[command(about, author, version)]
pub struct Args {
    // General config
    /// The IP address and port to handle requests on.
    #[arg(
        short,
        long,
        env = "DUMB_AUTH_BIND_ADDR",
        hide_env = true,
        default_value = "0.0.0.0:3862"
    )]
    pub bind_addr: SocketAddr,

    // Auth config
    /// The password required to login.
    #[arg(short, long, env = "DUMB_AUTH_PASSWORD", hide_env = true)]
    pub password: String,
    /// Support HTTP Basic authentication.
    #[arg(long, env = "DUMB_AUTH_ALLOW_BASIC", hide_env = true)]
    pub allow_basic: bool,
    /// Support HTTP Bearer token authentication.
    #[arg(long, env = "DUMB_AUTH_ALLOW_BEARER", hide_env = true)]
    pub allow_bearer: bool,
    /// Support interactive session/cookie authentication.
    #[arg(
        long,
        env = "DUMB_AUTH_ALLOW_SESSION",
        hide_env = true,
        action = ArgAction::Set,
        default_value_t = true,
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "true",
    )]
    pub allow_session: bool,

    // Session config
    /// The name of the session cookie to use.
    #[arg(
        long,
        env = "DUMB_AUTH_SESSION_COOKIE_NAME",
        hide_env = true,
        default_value = AuthConfig::DEFAULT_SESSION_COOKIE_NAME
    )]
    pub session_cookie_name: String,
    /// The domain to set the session cookie on.
    ///
    /// Leave this unset if you only have a single domain,
    /// or you want each domain to have a separate session. Otherwise set it to your parent domain,
    /// e.g. `example.com`, to have sessions shared across all subdomains, i.e. if you want
    /// `a.example.com` and `b.example.com` to share the same session.
    #[arg(
        short = 'd',
        long,
        env = "DUMB_AUTH_SESSION_COOKIE_DOMAIN",
        hide_env = true
    )]
    pub session_cookie_domain: Option<String>,
    #[arg(
        long,
        env = "DUMB_AUTH_SESSION_EXPIRY",
        hide_env = true,
        default_value_t = AuthConfig::DEFAULT_SESSION_EXPIRY
    )]
    /// How long after creation a session should expire.
    ///
    /// One of:
    ///
    /// "never": Sessions don't expire. Realistically sessions will expired with `dumb-auth` is
    /// restarted, since session are current stored in memory.
    ///
    /// "session": Sessions expire when the browser decides that it's "session" has ended. This is
    /// up to the browser.
    ///
    /// A duration: A fixed duration, e.g. `7d`, `1d12h`, `1week 2days 3hours 4minutes`, etc.
    pub session_expiry: SessionExpiry,
}

impl From<Args> for AuthConfig {
    fn from(args: Args) -> Self {
        Self {
            password: args.password,
            allow_basic: args.allow_basic,
            allow_bearer: args.allow_bearer,
            allow_session: args.allow_session,
            session_cookie_name: args.session_cookie_name,
            session_cookie_domain: args.session_cookie_domain,
            session_expiry: args.session_expiry,
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();
    let listener = TcpListener::bind(&args.bind_addr).await.unwrap();
    let app = dumb_auth::app(args.into());
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use std::{env, ffi::OsString, iter};

    use clap::CommandFactory;

    use super::*;

    #[test]
    fn verify_args() {
        Args::command().debug_assert();
    }

    #[test]
    fn test_allow_session() {
        // Default
        assert_eq!(args_min(iter::empty::<&str>()).unwrap().allow_session, true);
        assert_eq!(args_min(["--allow-session"]).unwrap().allow_session, true);

        // true
        assert_eq!(
            args_min(["--allow-session=true"]).unwrap().allow_session,
            true
        );

        // false
        assert_eq!(
            args_min(["--allow-session=false"]).unwrap().allow_session,
            false
        );

        // env
        env::set_var("DUMB_AUTH_ALLOW_SESSION", "false");
        assert_eq!(
            args_min(iter::empty::<&str>()).unwrap().allow_session,
            false
        );
        env::set_var("DUMB_AUTH_ALLOW_SESSION", "true");
        assert_eq!(args_min(iter::empty::<&str>()).unwrap().allow_session, true);
    }

    fn args_min<I, T>(itr: I) -> Result<Args, String>
    where
        I: IntoIterator<Item = T>,
        T: From<&'static str> + Into<OsString> + Clone,
    {
        args(iter::once("--password=hunter2".into()).chain(itr))
    }

    fn args<I, T>(itr: I) -> Result<Args, String>
    where
        I: IntoIterator<Item = T>,
        T: From<&'static str> + Into<OsString> + Clone,
    {
        Args::try_parse_from(iter::once("dumb-auth".into()).chain(itr)).map_err(|e| e.to_string())
    }
}
