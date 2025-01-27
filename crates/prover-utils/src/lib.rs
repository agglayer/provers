use std::str::FromStr;

pub mod with;

/// Get an environment variable or a default value if it is not set.
pub fn from_env_or_default<T: FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}
