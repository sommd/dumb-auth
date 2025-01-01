use crate::config::AuthConfig;

pub fn check_password(auth_config: &AuthConfig, password: &str) -> bool {
    constant_time_eq::constant_time_eq(auth_config.password.as_bytes(), password.as_bytes())
}
