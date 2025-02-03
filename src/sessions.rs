use std::{sync::Arc, time::SystemTime};

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::{
    config::SessionExpiry,
    datastore::{Datastore, StoreSessionError},
    AppError,
};

pub(crate) struct SessionStore {
    expiry: SessionExpiry,
    datastore: Arc<dyn Datastore>,
}

impl SessionStore {
    pub fn new(expiry: SessionExpiry, datastore: Arc<dyn Datastore>) -> Self {
        Self { expiry, datastore }
    }

    pub async fn get_valid_session(&self, token: &str) -> Result<Option<Session>, AppError> {
        let session = match self.datastore.get_session(token).await? {
            Some(session) => session,
            None => return Ok(None),
        };

        if let SessionExpiry::Duration(expiry) = self.expiry {
            if session.created.elapsed().unwrap_or_default() >= expiry {
                self.datastore.delete_session(token).await?;
                return Ok(None);
            }
        }

        Ok(Some(session))
    }

    pub async fn create_session(&self) -> Result<(String, Session), AppError> {
        let session = Session {
            created: SystemTime::now(),
        };

        let token = loop {
            let token = generate_token();
            match self.datastore.store_session(&token, &session).await? {
                Ok(()) => break token,
                Err(StoreSessionError::AlreadyExists) => continue,
            }
        };

        Ok((token, session))
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Session {
    pub created: SystemTime,
}

fn generate_token() -> String {
    thread_rng()
        .sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect::<String>()
}
