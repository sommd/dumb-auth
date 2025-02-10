use std::{fs::File, path::Path};

use heed::{
    byteorder::{BigEndian, NativeEndian},
    types::{SerdeBincode, Str, U64},
    Database, Env, EnvFlags, EnvOpenOptions, RoTxn, RwTxn,
};
use tokio::task::spawn_blocking;

use crate::sessions::{SessionData, SessionId};

use super::{DatastoreError, Result};

pub struct LmdbDatastore {
    env: Env,
    default: Database<Str, U64<NativeEndian>>,
    sessions: Database<U64<BigEndian>, SerdeBincode<SessionData>>,
}

impl LmdbDatastore {
    const MARKER: u64 = 0x64756d6261757468;
    const VERSION: u64 = 1;

    const SESSIONS_DB_NAME: &str = "sessions";

    const MARKER_KEY: &str = "dumb-auth-datastore";
    const VERSION_KEY: &str = "version";
    const SESSION_ID_COUNTER_KEY: &str = "session-id-counter";

    pub fn open(path: &Path) -> Result<Self> {
        let is_new = match File::options()
            .create(true)
            .write(true)
            .truncate(false)
            .open(path)
        {
            Ok(f) => f.metadata().map_err(heed::Error::Io)?.len() == 0,
            Err(e) => return Err(heed::Error::Io(e).into()),
        };

        let env = unsafe {
            EnvOpenOptions::new()
                .max_dbs(2)
                .map_size(4 * 1024 * 1024)
                .flags(EnvFlags::NO_SUB_DIR)
                .open(path)?
        };

        if is_new {
            Self::init(env)
        } else {
            Self::check(env)
        }
    }

    fn init(env: Env) -> Result<Self> {
        let mut wtxn = env.write_txn()?;

        // Create DBs
        let default = env
            .open_database(&wtxn, None)?
            .expect("default database should exist");
        let sessions = env.create_database(&mut wtxn, Some(Self::SESSIONS_DB_NAME))?;

        // Create metadata
        default.put(&mut wtxn, Self::MARKER_KEY, &Self::MARKER)?;
        default.put(&mut wtxn, Self::VERSION_KEY, &Self::VERSION)?;
        default.put(&mut wtxn, Self::SESSION_ID_COUNTER_KEY, &1)?;

        wtxn.commit()?;

        Ok(Self {
            env,
            default,
            sessions,
        })
    }

    fn check(env: Env) -> Result<Self> {
        let rtxn = env.read_txn()?;

        let default = env
            .open_database(&rtxn, None)?
            .expect("default database should exist");

        // Check marker
        if default.get(&rtxn, Self::MARKER_KEY)? != Some(Self::MARKER) {
            return Err(DatastoreError::UnrecognizedFormat);
        }

        // Check version
        match default.get(&rtxn, Self::VERSION_KEY)? {
            Some(Self::VERSION) => {}
            Some(version) => return Err(DatastoreError::UnknownVersion(version)),
            None => return Err(DatastoreError::Corrupt),
        };

        // Check other metadata
        default
            .get(&rtxn, Self::SESSION_ID_COUNTER_KEY)?
            .ok_or(DatastoreError::Corrupt)?;

        // Open databases
        let sessions = env
            .open_database(&rtxn, Some(Self::SESSIONS_DB_NAME))?
            .ok_or(DatastoreError::Corrupt)?;

        rtxn.commit()?;

        Ok(Self {
            env,
            default,
            sessions,
        })
    }

    pub async fn create_session(&self, data: SessionData) -> Result<SessionId> {
        let default = self.default;
        let sessions = self.sessions;
        self.write(move |wtxn| {
            // Generate ID
            let id = default
                .get(wtxn, Self::SESSION_ID_COUNTER_KEY)?
                .ok_or(DatastoreError::Corrupt)?;
            default.put(wtxn, Self::SESSION_ID_COUNTER_KEY, &(id + 1))?;

            // Write session
            sessions.put(wtxn, &id, &data)?;

            Ok(SessionId(id))
        })
        .await
    }

    pub async fn read_session(&self, id: SessionId) -> Result<Option<SessionData>> {
        let sessions = self.sessions;
        self.read(move |rtxn| Ok(sessions.get(rtxn, &id.0)?)).await
    }

    pub async fn delete_session(&self, id: SessionId) -> Result<bool> {
        let sessions = self.sessions;
        self.write(move |wtxn| Ok(sessions.delete(wtxn, &id.0)?))
            .await
    }

    async fn read<T: Send + 'static>(
        &self,
        op: impl FnOnce(&RoTxn) -> Result<T> + Send + 'static,
    ) -> Result<T> {
        let rtxn = self.env.read_txn()?;
        let ret = op(&rtxn)?;
        rtxn.commit()?;
        Ok(ret)
    }

    async fn write<T: Send + 'static>(
        &self,
        op: impl FnOnce(&mut RwTxn) -> Result<T> + Send + 'static,
    ) -> Result<T> {
        let env = self.env.clone();
        spawn_blocking(move || {
            let mut wtxn = env.write_txn()?;
            let ret = op(&mut wtxn)?;
            wtxn.commit()?;
            Ok(ret)
        })
        .await
        .unwrap()
    }
}
