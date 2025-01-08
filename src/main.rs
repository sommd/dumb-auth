use std::{fs::File, io::Write, net::SocketAddr, path::PathBuf};

use clap::{ArgAction, Args, Parser, Subcommand};
use tokio::net::TcpListener;
use zeroize::Zeroize;

use dumb_auth::{AuthConfig, Password, SessionExpiry};

#[derive(Debug, PartialEq, Parser)]
#[command(about, author, version, args_conflicts_with_subcommands = true)]
struct Cli {
    #[command(flatten)]
    args: Option<RunArgs>,

    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Debug, PartialEq, Subcommand)]
enum Cmd {
    // Run(RunArgs),
    Passwd(PasswdArgs),
}

#[derive(Debug, PartialEq, Args)]
struct RunArgs {
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

/// Hash a password, to be used with --password-hash[-file].
///
/// Password hashing follows the recommended OWASP password guidelines as of January 2025.
#[derive(Debug, PartialEq, Args)]
struct PasswdArgs {
    /// File to output password to instead of stdout, or "-" for stdout. File will be overwritten.
    #[arg(hide_default_value = true, default_value = "-")]
    output: PathBuf,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = Cli::parse();

    match cli.cmd {
        None => run(cli.args.unwrap()).await,
        Some(Cmd::Passwd(args)) => passwd(args),
    };
}

async fn run(args: RunArgs) {
    let listener = TcpListener::bind(&args.bind_addr).await.unwrap();

    let app = dumb_auth::app(AuthConfig {
        password: Password::Plain(args.password),
        allow_basic: args.allow_basic,
        allow_bearer: args.allow_bearer,
        allow_session: args.allow_session,
        session_cookie_name: args.session_cookie_name,
        session_cookie_domain: args.session_cookie_domain,
        session_expiry: args.session_expiry,
    });

    axum::serve(listener, app).await.unwrap();
}

fn passwd(args: PasswdArgs) {
    let mut plain = rpassword::prompt_password("Enter password: ").unwrap();
    let hash = dumb_auth::hash_password(&plain).unwrap();
    plain.zeroize();

    if args.output.to_str() == Some("-") {
        println!("{}", &hash);
    } else {
        writeln!(&mut File::create(args.output).unwrap(), "{}", &hash).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::{env, iter};

    use clap::CommandFactory;

    use super::*;

    fn sut(args: &[&str]) -> Result<Cli, String> {
        Cli::try_parse_from(iter::once(&"dumb-auth").chain(args)).map_err(|e| e.to_string())
    }

    #[test]
    fn verify_args() {
        Cli::command().debug_assert();
    }

    #[test]
    fn test_allow_session() {
        // Default
        assert_eq!(sut(&["-p=fake"]).unwrap().args.unwrap().allow_session, true);
        assert_eq!(
            sut(&["-p=fake", "--allow-session"])
                .unwrap()
                .args
                .unwrap()
                .allow_session,
            true
        );

        // Explicit
        assert_eq!(
            sut(&["-p=fake", "--allow-session=true"])
                .unwrap()
                .args
                .unwrap()
                .allow_session,
            true
        );
        assert_eq!(
            sut(&["-p=fake", "--allow-session=false"])
                .unwrap()
                .args
                .unwrap()
                .allow_session,
            false
        );

        // Env
        env::set_var("DUMB_AUTH_ALLOW_SESSION", "false");
        assert_eq!(
            sut(&["-p=fake"]).unwrap().args.unwrap().allow_session,
            false
        );
        env::set_var("DUMB_AUTH_ALLOW_SESSION", "true");
        assert_eq!(sut(&["-p=fake"]).unwrap().args.unwrap().allow_session, true);
        env::remove_var("DUMB_AUTH_ALLOW_SESSION");
    }

    #[test]
    fn test_passwd() {
        assert_eq!(
            sut(&["passwd"]).unwrap().cmd.unwrap(),
            Cmd::Passwd(PasswdArgs { output: "-".into() })
        );
        assert_eq!(
            sut(&["passwd", "outfile"]).unwrap().cmd.unwrap(),
            Cmd::Passwd(PasswdArgs {
                output: "outfile".into()
            })
        );

        assert!(sut(&["-p=fake", "passwd"])
            .unwrap_err()
            .contains("subcommand 'passwd' cannot be used with '--password"));
    }
}
