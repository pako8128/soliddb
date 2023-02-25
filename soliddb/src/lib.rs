#![deny(missing_docs)]

//! This crate provides traits for storing serializable types
//! in RocksDB.

mod error;
mod index;
mod iter;
mod keys;
mod single;
mod table;

pub use error::{Error, Result};
pub use index::IndexValue;
pub use iter::Items;
pub use single::Single;
pub use table::{Table, WithId};

pub use rocksdb::DB;
pub use soliddb_derive::{Single, Table};

/// Opens a new RocksDB Database at the given path.
///
/// This function ensures that the directory exists before
/// opening the database with default parameters.
pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<DB> {
    std::fs::create_dir_all(path.as_ref()).map_err(Error::CreateDirectory)?;
    let db = DB::open_default(path.as_ref())?;
    Ok(db)
}
