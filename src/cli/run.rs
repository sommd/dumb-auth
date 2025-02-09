use std::{fs, net::SocketAddr, path::PathBuf};

use clap::{ArgAction, Args};
use dumb_auth::{AppConfig, AuthConfig, Datastore, Password, SessionExpiry};
use password_hash::PasswordHashString;
use tokio::{net::TcpListener, runtime::Runtime};
use tracing::info;

use super::common::{die, fatal};

#[derive(Args, Debug, PartialEq)]
pub struct RunArgs {
    // Server
    /// The IP address and port to handle requests on.
    #[arg(
        short,
        long,
        env = "DUMB_AUTH_BIND_ADDR",
        hide_env = true,
        default_value = "0.0.0.0:3862"
    )]
    pub bind_addr: SocketAddr,
    /// The base path for public routes. Must start with `/`.
    #[arg(
        long,
        env = "DUMB_AUTH_PUBLIC_PATH",
        hide_env = true,
        value_parser = parse_base_path,
        default_value = AppConfig::DEFAULT_PUBLIC_PATH
    )]
    pub public_path: String,

    // Runtime
    /// Number of worker threads to use. 0 to detect and use the number of CPU cores/threads (the
    /// default).
    #[arg(
        short = 'T',
        long,
        env = "DUMB_AUTH_THREADS",
        hide_env = true,
        default_value_t = 0,
        hide_default_value = true,
        group = "threads_arg"
    )]
    pub threads: usize,
    /// Use only a single thread with no workers.
    ///
    /// This should use much less memory and can still handle multiple requests concurrently, but
    /// may not be quite as performant as using '--threads'.
    #[arg(
        long,
        env = "DUMB_AUTH_SINGLE_THREAD",
        hide_env = true,
        group = "threads_arg"
    )]
    pub single_thread: bool,

    // Password
    /// The password (in plain text) used to authenticate.
    #[arg(
        long,
        env = "DUMB_AUTH_PASSWORD",
        hide_env = true,
        group = "password_arg",
        required = true
    )]
    pub password: Option<String>,
    /// A file containing the password (in plain text) used to authenticate.
    #[arg(
        long,
        env = "DUMB_AUTH_PASSWORD_FILE",
        hide_env = true,
        group = "password_arg",
        required = true
    )]
    pub password_file: Option<PathBuf>,
    /// The hash of the password used to authenticate. Use the 'passwd' subcommand to generate the
    /// hash.
    #[arg(
        long,
        env = "DUMB_AUTH_PASSWORD_HASH",
        hide_env = true,
        group = "password_arg",
        required = true
    )]
    pub password_hash: Option<String>,
    /// A file containing the hash of the password used to authenticate. Use the 'passwd' subcommand
    /// to generate the hash.
    #[arg(
        long,
        env = "DUMB_AUTH_PASSWORD_HASH_FILE",
        hide_env = true,
        group = "password_arg",
        required = true
    )]
    pub password_hash_file: Option<PathBuf>,

    // Methods
    /// Support HTTP Basic authentication.
    #[arg(long, env = "DUMB_AUTH_ALLOW_BASIC", hide_env = true)]
    pub allow_basic: bool,
    /// Support HTTP Bearer token authentication.
    #[arg(long, env = "DUMB_AUTH_ALLOW_BEARER", hide_env = true)]
    pub allow_bearer: bool,
    /// Support session (interactive) authentication.
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

    // Session
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
    #[arg(
        long,
        env = "DUMB_AUTH_SESSION_EXPIRY",
        hide_env = true,
        default_value_t = AuthConfig::DEFAULT_SESSION_EXPIRY
    )]
    pub session_expiry: SessionExpiry,

    // Datastore
    #[arg(long, env = "DUMB_AUTH_DATASTORE", hide_env = true)]
    pub datastore: Option<PathBuf>,
}

impl RunArgs {
    pub fn runtime(&self) -> Runtime {
        let mut builder = if self.single_thread {
            tokio::runtime::Builder::new_current_thread()
        } else {
            tokio::runtime::Builder::new_multi_thread()
        };

        if self.threads != 0 {
            builder.worker_threads(self.threads);
        }

        builder
            .enable_all()
            .build()
            .unwrap_or_else(|e| fatal("creating runtime", e))
    }

    pub fn password(&self) -> Password {
        let read_file = |path| {
            let mut string =
                fs::read_to_string(path).unwrap_or_else(|e| fatal("reading password/hash file", e));

            // Trim final \n or \r\n
            if string.ends_with('\n') {
                string.pop();
                if string.ends_with('\r') {
                    string.pop();
                }
            }

            string
        };

        let parse_hash = |hash| {
            PasswordHashString::new(hash).unwrap_or_else(|e| fatal("parsing password hash", e))
        };

        let password = if let Some(plain) = &self.password {
            Password::Plain(plain.clone())
        } else if let Some(path) = &self.password_file {
            Password::Plain(read_file(path))
        } else if let Some(hash) = &self.password_hash {
            Password::Hash(parse_hash(hash))
        } else if let Some(path) = &self.password_hash_file {
            Password::Hash(parse_hash(&read_file(path)))
        } else {
            unreachable!()
        };

        if let Password::Plain(password) = &password {
            if password.is_empty() {
                die("Password cannot be empty");
            }
        }

        password
    }

    pub async fn datastore(&self) -> Datastore {
        match &self.datastore {
            Some(path) => Datastore::open(path).unwrap_or_else(|e| fatal("opening datastore", e)),
            None => Datastore::new_in_memory(),
        }
    }
}

fn parse_base_path(s: &str) -> Result<String, String> {
    if s.is_empty() {
        Err("base path must not be empty".into())
    } else if !s.starts_with('/') {
        Err("base path must start with '/'".into())
    } else if s.len() > 1 && s.ends_with('/') {
        Err("base path must not end with '/'".into())
    } else if s.contains("//") {
        Err("base path must not contain '//'".into())
    } else if s.contains(".") {
        Err("base path must not contain '.'".into())
    } else if s.contains("{") {
        Err("base path must not contain '{'".into())
    } else if s.contains("}") {
        Err("base path must not contain '}'".into())
    } else {
        Ok(s.into())
    }
}

pub fn run(args: RunArgs) {
    args.runtime().block_on(async {
        let password = args.password();
        let datastore = args.datastore().await;
        let config = dumb_auth::AppConfig {
            public_path: args.public_path,
            auth_config: AuthConfig {
                password,
                allow_basic: args.allow_basic,
                allow_bearer: args.allow_bearer,
                allow_session: args.allow_session,
                session_cookie_name: args.session_cookie_name,
                session_cookie_domain: args.session_cookie_domain,
                session_expiry: args.session_expiry,
            },
        };

        let app = dumb_auth::app(config, datastore);

        let listener = TcpListener::bind(&args.bind_addr).await.unwrap();
        info!("Listening for requests on http://{}", &args.bind_addr);
        axum::serve(listener, app).await.unwrap();
    });
}
