use async_trait::async_trait;
use heed::{
    types::{SerdeBincode, Str},
    Database, Env,
};

use crate::sessions::Session;

use super::{Datastore, DatastoreError, StoreSessionError};

pub struct LmdbDatastore {
    env: Env,
    sessions: Database<Str, SerdeBincode<Session>>,
}

impl LmdbDatastore {
    pub fn init(env: Env) -> heed::Result<Self> {
        let mut wtxn = env.write_txn()?;

        let sessions = env.create_database(&mut wtxn, Some("sessions"))?;

        wtxn.commit()?;

        Ok(Self { env, sessions })
    }
}

#[async_trait]
impl Datastore for LmdbDatastore {
    async fn store_session(
        &self,
        token: &str,
        session: &Session,
    ) -> Result<Result<(), StoreSessionError>, DatastoreError> {
        let mut wtxn = self.env.write_txn()?;

        if self
            .sessions
            .get_or_put(&mut wtxn, token, session)?
            .is_none()
        {
            wtxn.commit()?;

            Ok(Ok(()))
        } else {
            Ok(Err(StoreSessionError::AlreadyExists))
        }
    }

    async fn get_session(&self, token: &str) -> Result<Option<Session>, DatastoreError> {
        let rtxn = self.env.read_txn()?;
        Ok(self.sessions.get(&rtxn, token)?)
    }

    async fn delete_session(&self, token: &str) -> Result<bool, DatastoreError> {
        let mut wtxn = self.env.write_txn()?;

        if self.sessions.delete(&mut wtxn, token)? {
            wtxn.commit()?;

            Ok(true)
        } else {
            Ok(false)
        }
    }
}
