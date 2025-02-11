use dumb_auth::Datastore;

#[path = "."]
mod in_memory {
    use super::*;

    fn create_datastore() -> (Datastore, ()) {
        (Datastore::new_in_memory(), ())
    }

    #[path = "integration/mod.rs"]
    mod integration;
}

#[path = "."]
mod lmdb {
    use dumb_auth::{ReadMode, WriteMode};
    use tempfile::TempDir;

    use super::*;

    fn create_datastore_with(read_mode: ReadMode, write_mode: WriteMode) -> (Datastore, TempDir) {
        let dir = TempDir::new().unwrap();
        let datastore =
            Datastore::open_with(dir.path().join("dumb-auth.mdb"), read_mode, write_mode).unwrap();
        (datastore, dir)
    }

    #[path = "."]
    mod sync {
        use super::*;

        fn create_datastore() -> (Datastore, TempDir) {
            create_datastore_with(ReadMode::Sync, WriteMode::Sync)
        }

        #[path = "integration/mod.rs"]
        mod integration;
    }

    #[path = "."]
    mod async_ {
        use super::*;

        fn create_datastore() -> (Datastore, TempDir) {
            create_datastore_with(ReadMode::Async, WriteMode::Async)
        }

        #[path = "integration/mod.rs"]
        mod integration;
    }

    #[path = "."]
    mod async_thread {
        use super::*;

        fn create_datastore() -> (Datastore, TempDir) {
            create_datastore_with(ReadMode::Async, WriteMode::AsyncThread)
        }

        #[path = "integration/mod.rs"]
        mod integration;
    }
}
