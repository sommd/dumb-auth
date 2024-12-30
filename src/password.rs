use crate::config::Config;

pub fn check_password(config: &Config, password: &str) -> bool {
    constant_time_eq::constant_time_eq(config.password.as_bytes(), password.as_bytes())
}
