use std::collections::HashMap;

use argon2::Argon2;
use password_hash::{PasswordHashString, PasswordHasher, PasswordVerifier, SaltString};
use rand::thread_rng;
use subtle::ConstantTimeEq;
use tokio::sync::RwLock;

use crate::config::Password;

#[derive(Default)]
pub struct PasswordChecker {
    hash_cache: RwLock<HashMap<String, String>>,
}

impl PasswordChecker {
    pub async fn check_password(&self, input: &str, configured: &Password) -> bool {
        match configured {
            Password::Plain(configured) => verify_password(input, configured),
            Password::Hash(configured) => {
                if let Some(result) = self.check_cache(input, configured).await {
                    result
                } else {
                    self.check_hash(input, configured).await
                }
            }
        }
    }

    async fn check_cache(&self, input: &str, configured: &PasswordHashString) -> Option<bool> {
        let hash_cache = self.hash_cache.read().await;
        let cached = hash_cache.get(configured.as_str())?;
        Some(verify_password(input, cached))
    }

    async fn check_hash(&self, input: &str, configured: &PasswordHashString) -> bool {
        if verify_hash(input, configured) {
            self.hash_cache
                .write()
                .await
                .entry(configured.as_str().into())
                .or_insert_with(|| input.into());

            true
        } else {
            false
        }
    }
}

fn verify_password(a: &str, b: &str) -> bool {
    a.as_bytes().ct_eq(b.as_bytes()).into()
}

fn verify_hash(password: &str, hash: &PasswordHashString) -> bool {
    password_hasher()
        .verify_password(password.as_bytes(), &hash.password_hash())
        .is_ok()
}

pub fn hash_password(input: &str) -> password_hash::Result<PasswordHashString> {
    let salt = SaltString::generate(thread_rng());
    let hash = password_hasher().hash_password(input.as_bytes(), &salt)?;
    Ok(hash.serialize())
}

fn password_hasher() -> impl PasswordHasher {
    Argon2::default()
}
