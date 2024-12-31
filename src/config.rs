use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
};

use clap::Parser;
use duration_str::HumanFormat;
use time::Duration;

const DEFAULT_BIND_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 3862);
const DEFAULT_SESSION_COOKIE_NAME: &str = "dumb-auth-session";
const DEFAULT_SESSION_EXPIRY: SessionExpiry = SessionExpiry::Duration(Duration::weeks(4));

#[derive(Clone, Debug, Parser)]
pub struct Config {
    // General config
    /// The IP address and port to handle requests on.
    #[arg(
        short,
        long,
        env = "DUMB_AUTH_BIND_ADDR",
        hide_env = true,
        default_value_t = DEFAULT_BIND_ADDR
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

    // Session config
    /// The name of the session cookie to use.
    #[arg(
        long,
        env = "DUMB_AUTH_SESSION_COOKIE_NAME",
        hide_env = true,
        default_value = DEFAULT_SESSION_COOKIE_NAME
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
        default_value_t = DEFAULT_SESSION_EXPIRY
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

impl Config {
    pub fn new(password: &str) -> Self {
        Self {
            bind_addr: DEFAULT_BIND_ADDR,
            password: password.into(),
            allow_basic: Default::default(),
            allow_bearer: Default::default(),
            session_cookie_name: DEFAULT_SESSION_COOKIE_NAME.into(),
            session_cookie_domain: Default::default(),
            session_expiry: DEFAULT_SESSION_EXPIRY,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SessionExpiry {
    Session,
    Duration(Duration),
}

impl fmt::Display for SessionExpiry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SessionExpiry::Session => write!(f, "session"),
            SessionExpiry::Duration(duration) => write!(f, "{}", duration.human_format()),
        }
    }
}

impl FromStr for SessionExpiry {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "session" => Ok(Self::Session),
            _ => duration_str::parse_time(s).map(Self::Duration),
        }
    }
}
