use dumb_auth::Datastore;

#[path = "."]
mod inmemory {
    use super::*;

    fn create_datastore() -> (Datastore, ()) {
        (Datastore::new_in_memory(), ())
    }

    #[path = "integration/mod.rs"]
    mod integration;
}

#[path = "."]
mod lmdb {
    use tempfile::TempDir;

    use super::*;

    fn create_datastore() -> (Datastore, TempDir) {
        let dir = TempDir::new().unwrap();
        let datastore = Datastore::open(dir.path().join("dumb-auth.mdb")).unwrap();
        (datastore, dir)
    }

    #[path = "integration/mod.rs"]
    mod integration;
}
