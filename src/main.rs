use std::{
    fs::{self, File},
    io::{self, Write},
    net::SocketAddr,
    path::PathBuf,
    process,
};

use clap::{ArgAction, Args, Parser, Subcommand};
use dumb_auth::{AppConfig, AuthConfig, Password, SessionExpiry};
use password_hash::PasswordHashString;
use tokio::{net::TcpListener, runtime};
use tracing::{error, info, level_filters::LevelFilter};
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use zeroize::Zeroize;

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

#[derive(Args, Debug, PartialEq)]
struct RunArgs {
    // General
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

impl RunArgs {
    pub fn password(&self) -> Result<Password, PasswordError> {
        let password = if let Some(plain) = &self.password {
            Password::Plain(plain.into())
        } else if let Some(path) = &self.password_file {
            Password::Plain(Self::read_file(path)?)
        } else if let Some(hash) = &self.password_hash {
            Password::Hash(Self::parse_hash(hash)?)
        } else if let Some(path) = &self.password_hash_file {
            Password::Hash(Self::parse_hash(&Self::read_file(path)?)?)
        } else {
            unreachable!()
        };

        if let Password::Plain(plain) = &password {
            if plain.is_empty() {
                return Err(PasswordError::Empty);
            }
        }

        Ok(password)
    }

    fn read_file(path: &PathBuf) -> Result<String, PasswordError> {
        let mut string = fs::read_to_string(path)?;

        // Trim final \n or \r\n
        if string.ends_with('\n') {
            string.pop();
            if string.ends_with('\r') {
                string.pop();
            }
        }

        Ok(string)
    }

    fn parse_hash(hash: &str) -> Result<PasswordHashString, PasswordError> {
        Ok(PasswordHashString::new(hash)?)
    }
}

enum PasswordError {
    Io(io::Error),
    Hash(password_hash::Error),
    Empty,
}

impl From<io::Error> for PasswordError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<password_hash::Error> for PasswordError {
    fn from(value: password_hash::Error) -> Self {
        Self::Hash(value)
    }
}

/// Hash a password, to be used with --password-hash[-file].
///
/// Password hashing follows the recommended OWASP password guidelines as of January 2025.
#[derive(Args, Debug, PartialEq)]
struct PasswdArgs {
    /// File to output password to instead of stdout, or "-" for stdout. File will be overwritten.
    #[arg(default_value = "-", hide_default_value = true)]
    output: PathBuf,
}

fn main() {
    FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::builder()
                .with_env_var("DUMB_AUTH_LOG")
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let cli = Cli::parse();
    match cli.cmd {
        None => run(cli.args.unwrap()),
        Some(Cmd::Passwd(args)) => passwd(args),
    };
}

fn run(args: RunArgs) {
    let password = args.password().unwrap_or_else(|e| {
        match e {
            PasswordError::Io(e) => error!("Error reading password/hash file: {}", e),
            PasswordError::Hash(e) => error!("Error parsing password hash: {}", e),
            PasswordError::Empty => error!("Password cannot be empty"),
        };
        process::exit(1);
    });

    let app = dumb_auth::app(dumb_auth::AppConfig {
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
    });

    let rt = {
        let mut builder = if args.single_thread {
            runtime::Builder::new_current_thread()
        } else {
            runtime::Builder::new_multi_thread()
        };

        builder.enable_all();
        if args.threads != 0 {
            builder.worker_threads(args.threads);
        }

        builder.build().unwrap_or_else(|e| {
            error!("Error creating runtime: {}", e);
            process::exit(1);
        })
    };

    rt.block_on(async {
        let listener = TcpListener::bind(&args.bind_addr).await.unwrap();
        info!("Listening for requests on http://{}", &args.bind_addr);
        axum::serve(listener, app).await.unwrap();
    });
}

fn passwd(args: PasswdArgs) {
    let mut plain = rpassword::prompt_password("Enter password: ").unwrap_or_else(|e| {
        error!("Error reading password: {}", e);
        process::exit(1);
    });

    let hash = dumb_auth::hash_password(&plain).unwrap_or_else(|e| {
        error!("Error hashing password: {}", e);
        process::exit(1);
    });

    plain.zeroize();

    if args.output.to_str() == Some("-") {
        println!("{}", &hash);
    } else {
        File::create(args.output)
            .and_then(|mut f| writeln!(f, "{}", hash))
            .unwrap_or_else(|e| {
                error!("Error writing to output file: {}", e);
                process::exit(1);
            });
    }
}

#[cfg(test)]
mod tests {
    use std::{env, iter};

    use clap::CommandFactory;

    use super::*;

    const PWARG: &str = "--password=hunter2";

    fn sut(args: &[&str]) -> Result<Cli, String> {
        Cli::try_parse_from(iter::once(&"dumb-auth").chain(args)).map_err(|e| e.to_string())
    }

    #[test]
    fn verify_args() {
        Cli::command().debug_assert();
    }

    #[test]
    fn test_threads() {
        // Disallows both --threads and --single-thread
        assert!(sut(&[PWARG, "--threads=2", "--single-thread"])
            .unwrap_err()
            .contains("cannot be used with '--single-thread"));
    }

    #[test]
    fn test_password() {
        // Requires a password arg
        assert!(sut(&[])
            .unwrap_err()
            .contains("required arguments were not provided"));

        // Accepts a single password arg
        assert_eq!(
            sut(&["--password=hunter2"])
                .unwrap()
                .args
                .unwrap()
                .password
                .as_deref(),
            Some("hunter2")
        );
        assert_eq!(
            sut(&["--password-file=password.txt"])
                .unwrap()
                .args
                .unwrap()
                .password_file,
            Some(PathBuf::from("password.txt"))
        );
        assert_eq!(
            sut(&["--password-hash=$1$n8iaq2sR$S2FSu61ixElrdPp/TUxtM0"])
                .unwrap()
                .args
                .unwrap()
                .password_hash
                .as_deref(),
            Some("$1$n8iaq2sR$S2FSu61ixElrdPp/TUxtM0")
        );
        assert_eq!(
            sut(&["--password-hash-file=password-hash.txt"])
                .unwrap()
                .args
                .unwrap()
                .password_hash_file,
            Some(PathBuf::from("password-hash.txt"))
        );

        // Disallows multiple password args
        assert!(sut(&["--password=hunter2", "--password-file=password.txt"])
            .unwrap_err()
            .contains("cannot be used with '--password"),);
    }

    #[test]
    fn test_allow_session() {
        // Defaults to true
        assert_eq!(sut(&[PWARG]).unwrap().args.unwrap().allow_session, true);
        assert_eq!(
            sut(&[PWARG, "--allow-session"])
                .unwrap()
                .args
                .unwrap()
                .allow_session,
            true
        );

        // Parses explicit value
        assert_eq!(
            sut(&[PWARG, "--allow-session=true"])
                .unwrap()
                .args
                .unwrap()
                .allow_session,
            true
        );
        assert_eq!(
            sut(&[PWARG, "--allow-session=false"])
                .unwrap()
                .args
                .unwrap()
                .allow_session,
            false
        );

        // Parses value from env
        env::set_var("DUMB_AUTH_ALLOW_SESSION", "false");
        assert_eq!(sut(&[PWARG]).unwrap().args.unwrap().allow_session, false);
        env::set_var("DUMB_AUTH_ALLOW_SESSION", "true");
        assert_eq!(sut(&[PWARG]).unwrap().args.unwrap().allow_session, true);
        env::remove_var("DUMB_AUTH_ALLOW_SESSION");
    }

    #[test]
    fn test_passwd() {
        // Does not require run args
        assert_eq!(
            sut(&["passwd"]).unwrap().cmd.unwrap(),
            Cmd::Passwd(PasswdArgs { output: "-".into() })
        );

        // Accepts outfile
        assert_eq!(
            sut(&["passwd", "outfile"]).unwrap().cmd.unwrap(),
            Cmd::Passwd(PasswdArgs {
                output: "outfile".into()
            })
        );

        // Disallows run args
        assert!(sut(&[PWARG, "passwd"])
            .unwrap_err()
            .contains("subcommand 'passwd' cannot be used with '--password"));
    }
}
