use std::{fmt, str::FromStr};

use duration_str::HumanFormat;
use time::Duration;

use crate::password::Password;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub public_path: String,
    pub auth_config: AuthConfig,
}

impl AppConfig {
    pub const DEFAULT_PUBLIC_PATH: &str = "/auth";

    pub fn default(auth_config: AuthConfig) -> Self {
        Self {
            public_path: Self::DEFAULT_PUBLIC_PATH.into(),
            auth_config,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AuthConfig {
    pub password: Password,
    pub allow_basic: bool,
    pub allow_bearer: bool,
    pub allow_session: bool,
    pub session_cookie_name: String,
    pub session_cookie_domain: Option<String>,
    pub session_expiry: SessionExpiry,
}

impl AuthConfig {
    pub const DEFAULT_SESSION_COOKIE_NAME: &'static str = "dumb-auth-session";
    pub const DEFAULT_SESSION_EXPIRY: SessionExpiry = SessionExpiry::Duration(Duration::weeks(4));

    pub fn default(password: Password) -> Self {
        Self {
            password,
            allow_basic: false,
            allow_bearer: false,
            allow_session: true,
            session_cookie_name: Self::DEFAULT_SESSION_COOKIE_NAME.to_string(),
            session_cookie_domain: None,
            session_expiry: Self::DEFAULT_SESSION_EXPIRY,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
