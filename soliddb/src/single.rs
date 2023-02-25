use rocksdb::DB;
use serde::{de::DeserializeOwned, Serialize};

use crate::table::{from_bytes, to_bytes};
use crate::{Error, Result};

/// Trait for storing a single instance of the given type
/// in a rocksdb database instance. Can be derived.
pub trait Single: Serialize + DeserializeOwned {
    /// Number uniquely identifying the type.
    const SINGLE: u32;

    /// Stores the value in the given db.
    fn put(&self, db: &DB) -> Result<()> {
        let serialized = to_bytes(self)?;
        db.put(key(Self::SINGLE), serialized)?;
        Ok(())
    }

    /// Retrieve the stored value from the given db.
    fn get(db: &DB) -> Result<Self> {
        let bytes = db.get_pinned(key(Self::SINGLE))?.ok_or(Error::NotFound)?;
        let value = from_bytes(&bytes)?;
        Ok(value)
    }

    /// Delete the stored value from the given db.
    fn delete(db: &DB) -> Result<()> {
        db.delete(key(Self::SINGLE))?;
        Ok(())
    }
}

fn key(single: u32) -> Vec<u8> {
    let mut key = vec![0; 4];
    key.extend_from_slice(&single.to_be_bytes());
    key
}
