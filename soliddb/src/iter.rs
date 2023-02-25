use std::marker::PhantomData;

use rocksdb::{DBIteratorWithThreadMode, DB};

use crate::{
    keys::{id_from_primary_key, key_prefix},
    table::from_bytes,
    Result, Table, WithId,
};

type KeyVal = (Box<[u8]>, Box<[u8]>);

pub(crate) struct PrefixIterator<'a> {
    prefix: Vec<u8>,
    inner: DBIteratorWithThreadMode<'a, DB>,
}

impl<'a> PrefixIterator<'a> {
    pub(crate) fn new(db: &'a DB, prefix: Vec<u8>) -> Self {
        let inner = db.prefix_iterator(&prefix);
        Self { prefix, inner }
    }
}

impl Iterator for PrefixIterator<'_> {
    type Item = Result<KeyVal>;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.inner.next()?;
        let (key, bytes) = match value {
            Ok(value) => value,
            Err(err) => return Some(Err(err.into())),
        };

        match key.starts_with(&self.prefix) {
            true => Some(Ok((key, bytes))),
            false => None,
        }
    }
}
/// Iterator of Items returned by [Table::iter](soliddb::Table::iter).
pub struct Items<'a, T> {
    inner: PrefixIterator<'a>,
    _marker: PhantomData<T>,
}

impl<'a, T: Table> Items<'a, T> {
    pub(crate) fn new(db: &'a DB) -> Self {
        let prefix = key_prefix(T::TABLE, 0);
        let inner = PrefixIterator::new(db, prefix);
        Self {
            inner,
            _marker: PhantomData,
        }
    }
}

impl<T: Table> Iterator for Items<'_, T> {
    type Item = Result<WithId<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(decode_item)
    }
}

fn decode_item<T: Table>(item: Result<KeyVal>) -> Result<WithId<T>> {
    let (key, val) = item?;
    let id = id_from_primary_key(&key)?;
    let value = from_bytes(&val)?;
    Ok(WithId { id, value })
}
