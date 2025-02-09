use std::{fs::File, path::Path};

use heed::{
    byteorder::{BigEndian, NativeEndian},
    types::{SerdeBincode, Str, U64},
    Database, Env, EnvFlags, EnvOpenOptions,
};

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
        let mut wtxn = self.env.write_txn()?;

        // Generate ID
        let id = self
            .default
            .get(&wtxn, Self::SESSION_ID_COUNTER_KEY)?
            .ok_or(DatastoreError::Corrupt)?;
        self.default
            .put(&mut wtxn, Self::SESSION_ID_COUNTER_KEY, &(id + 1))?;

        // Write session
        self.sessions.put(&mut wtxn, &id, &data)?;

        wtxn.commit()?;

        Ok(SessionId(id))
    }

    pub async fn read_session(&self, id: SessionId) -> Result<Option<SessionData>> {
        let rtxn = self.env.read_txn()?;

        Ok(self.sessions.get(&rtxn, &id.0)?)
    }

    pub async fn delete_session(&self, id: SessionId) -> Result<bool> {
        let mut wtxn = self.env.write_txn()?;

        let result = self.sessions.delete(&mut wtxn, &id.0)?;

        wtxn.commit()?;

        Ok(result)
    }
}
