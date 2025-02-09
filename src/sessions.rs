use std::{fmt, sync::Arc, time::SystemTime};

use base64ct::{Base64UrlUnpadded, Encoding};
use bincode::Options;
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use thiserror::Error;

use crate::{config::SessionExpiry, datastore::Datastore, AppError};

pub(crate) struct SessionManager {
    expiry: SessionExpiry,
    datastore: Arc<Datastore>,
}

impl SessionManager {
    pub fn new(expiry: SessionExpiry, datastore: Arc<Datastore>) -> Self {
        Self { expiry, datastore }
    }

    pub async fn create_session(&self) -> Result<SessionToken, AppError> {
        let secret = SessionSecret::generate();

        let id = self
            .datastore
            .create_session(SessionData {
                secret: secret.clone(),
                created: SystemTime::now(),
            })
            .await?;

        Ok(SessionToken { id, secret })
    }

    pub async fn check_session(&self, token: &str) -> Result<bool, AppError> {
        let token = match SessionToken::decode(token) {
            Ok(token) => token,
            Err(_) => return Ok(false),
        };

        let data = match self.datastore.read_session(token.id).await? {
            Some(data) => data,
            None => return Ok(false),
        };

        if !data.secret.verify(&token.secret) {
            return Ok(false);
        }

        if let SessionExpiry::Duration(expiry) = self.expiry {
            if data.created.elapsed().unwrap_or_default() >= expiry {
                self.datastore.delete_session(token.id).await?;
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SessionData {
    secret: SessionSecret,
    created: SystemTime,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SessionToken {
    id: SessionId,
    secret: SessionSecret,
}

impl SessionToken {
    pub fn decode(base64: &str) -> Result<Self, DecodeSessionTokenError> {
        let bytes = Base64UrlUnpadded::decode_vec(base64)?;
        let token = Self::bincode().deserialize(&bytes)?;
        Ok(token)
    }

    pub fn encode(&self) -> String {
        let bytes = Self::bincode()
            .serialize(self)
            .expect("session token should always be serializable");
        Base64UrlUnpadded::encode_string(&bytes)
    }

    fn bincode() -> impl bincode::Options {
        bincode::DefaultOptions::new()
    }
}

#[derive(Debug, Error)]
pub enum DecodeSessionTokenError {
    #[error("{0}")]
    Base64Error(#[from] base64ct::Error),
    #[error("{0}")]
    BincodeError(#[from] bincode::Error),
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct SessionId(pub u64);

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SessionSecret(Vec<u8>);

impl SessionSecret {
    const SIZE: usize = 32; // 256 bits

    pub fn generate() -> Self {
        let mut buf = vec![0u8; Self::SIZE];
        thread_rng().fill_bytes(&mut buf);
        Self(buf)
    }

    pub fn verify(&self, other: &Self) -> bool {
        self.0.ct_eq(&other.0).into()
    }
}

impl fmt::Debug for SessionSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SessionSecret").finish_non_exhaustive()
    }
}
