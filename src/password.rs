use std::{collections::HashMap, fmt};

use argon2::Argon2;
use password_hash::{PasswordHashString, PasswordHasher, PasswordVerifier, SaltString};
use rand::thread_rng;
use subtle::ConstantTimeEq;
use tokio::sync::RwLock;

#[derive(Clone)]
pub enum Password {
    Plain(String),
    Hash(PasswordHashString),
}

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plain(_) => f.debug_tuple("Plain").finish_non_exhaustive(),
            Self::Hash(_) => f.debug_tuple("Hash").finish_non_exhaustive(),
        }
    }
}

#[derive(Default)]
pub struct PasswordChecker {
    hash_cache: RwLock<HashMap<String, String>>,
}

impl PasswordChecker {
    pub async fn check_password(&self, input: &str, configured: &Password) -> bool {
        match configured {
            Password::Plain(configured) => verify_password(input, configured),
            Password::Hash(configured) => self.check_hashed(input, configured).await,
        }
    }

    async fn check_hashed(&self, input: &str, configured: &PasswordHashString) -> bool {
        let cache_key = configured.as_str();

        if let Some(cached) = self.hash_cache.read().await.get(cache_key) {
            // Check if cached password matches
            verify_password(input, cached)
        } else if verify_hashed(input, configured) {
            // Cache verified password
            self.hash_cache
                .write()
                .await
                .entry(cache_key.into())
                .or_insert_with(|| input.into());

            true
        } else {
            false
        }
    }
}

pub fn hash_password(input: &str) -> password_hash::Result<PasswordHashString> {
    let salt = SaltString::generate(thread_rng());
    let hash = password_hasher().hash_password(input.as_bytes(), &salt)?;
    Ok(hash.serialize())
}

fn verify_password(a: &str, b: &str) -> bool {
    a.as_bytes().ct_eq(b.as_bytes()).into()
}

fn verify_hashed(password: &str, hash: &PasswordHashString) -> bool {
    password_hasher()
        .verify_password(password.as_bytes(), &hash.password_hash())
        .is_ok()
}

fn password_hasher() -> impl PasswordHasher {
    Argon2::default()
}
