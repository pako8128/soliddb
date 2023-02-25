use rocksdb::{WriteBatch, DB};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use ulid::Ulid;

use crate::iter::PrefixIterator;
use crate::keys::{
    id_from_non_unique_key, id_from_slice, key_prefix, non_unique_key, primary_key, unique_key,
};
use crate::{Error, IndexValue, Items, Result};

/// Trait for storing a collection  of instances instance
/// of the given type in a rocksdb database instance. Can be derived.
pub trait Table: Serialize + DeserializeOwned {
    /// Number uniquely identifying the type.
    const TABLE: u32;

    /// List of unique indices.
    const UNIQUE_INDICES: &'static [u8] = &[];

    /// List of non-unique indices.
    const NON_UNIQUE_INDICES: &'static [u8] = &[];

    /// Returns a byte representation for the given unique index.
    fn unique_value(&self, index: u8) -> Vec<u8> {
        unreachable!("no unique value for index {index}")
    }

    /// Returns a byte representation for the given non-unique index.
    fn non_unique_value(&self, index: u8) -> Vec<u8> {
        unreachable!("no non-unique value for index {index}")
    }

    /// Storing this value in the given db returning the id.
    fn create(&self, db: &DB) -> Result<Ulid> {
        check_unique(db, self)?;

        let mut batch = WriteBatch::default();

        let id = Ulid::new();
        let key = primary_key(Self::TABLE, id);
        let serialized = to_bytes(self)?;

        batch.put(key, serialized);

        for index in Self::UNIQUE_INDICES {
            let value = self.unique_value(*index);
            let key = unique_key(Self::TABLE, *index, &value);
            batch.put(key, id.as_bytes());
        }

        for index in Self::NON_UNIQUE_INDICES {
            let value = self.non_unique_value(*index);
            let key = non_unique_key(Self::TABLE, *index, &value, id);
            batch.put(key, vec![]);
        }

        db.write(batch)?;
        Ok(id)
    }

    /// Returns the value for the given id.
    fn get(db: &DB, id: Ulid) -> Result<WithId<Self>> {
        let key = primary_key(Self::TABLE, id);
        let bytes = db.get_pinned(key)?.ok_or(Error::NotFound)?;
        let value = from_bytes(&bytes)?;
        Ok(WithId { id, value })
    }

    /// Returns the values for the given list of ids.
    fn get_many(db: &DB, ids: &[Ulid]) -> Result<Vec<WithId<Self>>> {
        let keys = ids.iter().map(|id| primary_key(Self::TABLE, *id));
        let values: Vec<_> = db
            .multi_get(keys)
            .into_iter()
            .collect::<std::result::Result<_, rocksdb::Error>>()?;

        let values: Vec<_> = values
            .into_iter()
            .map(|value| value.ok_or(Error::NotFound))
            .collect::<Result<_>>()?;

        let items = values
            .into_iter()
            .zip(ids.iter())
            .map(|(value, &id)| from_bytes(&value).map(|value| WithId { id, value }))
            .collect::<std::result::Result<_, ron::Error>>()?;

        Ok(items)
    }

    /// Returns the value for the given unique value.
    fn get_by_unique_index(db: &DB, index: u8, value: &[u8]) -> Result<WithId<Self>> {
        let key = unique_key(Self::TABLE, index, value);
        let id = db.get_pinned(key)?.ok_or(Error::NotFound)?;
        let id = id_from_slice(&id)?;
        Self::get(db, id)
    }

    /// Returns the values for the given non-unique values.
    fn get_by_non_unique_index(db: &DB, index: u8, value: &[u8]) -> Result<Vec<WithId<Self>>> {
        let mut prefix = key_prefix(Self::TABLE, index);
        prefix.extend_from_slice(value);
        let key_vals: Vec<_> = PrefixIterator::new(db, prefix).collect::<Result<_>>()?;
        let ids: Vec<_> = key_vals
            .into_iter()
            .map(|(key, _)| id_from_non_unique_key(&key))
            .collect::<Result<_>>()?;

        Self::get_many(db, &ids)
    }

    /// Returns an Iterator over all values of this type.
    fn iter(db: &DB) -> Items<Self> {
        Items::new(db)
    }

    /// Returns all values of this type.
    fn all(db: &DB) -> Result<Vec<WithId<Self>>> {
        Self::iter(db).collect()
    }

    /// Updating the entry for the given id with this value.
    fn update(&self, db: &DB, id: Ulid) -> Result<()> {
        let previous = Self::get(db, id)?;
        let key = primary_key(Self::TABLE, id);

        for index in Self::UNIQUE_INDICES {
            let unique_val = self.unique_value(*index);
            let key = unique_key(Self::TABLE, *index, &unique_val);
            let value = db.get(key)?;
            if value.is_some() && value != Some(id.as_bytes()) {
                return Err(Error::AlreadyExists);
            }
        }

        let mut batch = WriteBatch::default();

        let serialized = to_bytes(self)?;
        batch.put(key, serialized);

        for index in Self::UNIQUE_INDICES {
            let previous_value = previous.value.unique_value(*index);
            let new_value = self.unique_value(*index);

            if new_value != previous_value {
                let previous_key = unique_key(Self::TABLE, *index, &previous_value);
                let new_key = unique_key(Self::TABLE, *index, &new_value);
                batch.delete(previous_key);
                batch.put(new_key, id.as_bytes());
            }
        }

        for index in Self::NON_UNIQUE_INDICES {
            let previous_value = previous.value.non_unique_value(*index);
            let new_value = self.non_unique_value(*index);

            if new_value != previous_value {
                let previous_key = non_unique_key(Self::TABLE, *index, &previous_value, id);
                let new_key = non_unique_key(Self::TABLE, *index, &new_value, id);
                batch.delete(previous_key);
                batch.put(new_key, id.as_bytes());
            }
        }

        db.write(batch)?;
        Ok(())
    }

    /// Delete the entry for the given id.
    fn delete(db: &DB, id: Ulid) -> Result<()> {
        let item = Self::get(db, id)?;
        let key = primary_key(Self::TABLE, id);

        let mut batch = WriteBatch::default();
        batch.delete(key);

        for index in Self::UNIQUE_INDICES {
            let value = item.value.unique_value(*index);
            let key = unique_key(Self::TABLE, *index, &value);
            batch.delete(key);
        }

        for index in Self::NON_UNIQUE_INDICES {
            let value = item.value.non_unique_value(*index);
            let key = non_unique_key(Self::TABLE, *index, &value, id);
            batch.delete(key);
        }

        db.write(batch)?;
        Ok(())
    }
}

/// Wrapper type for an entries value and the associated id.
#[derive(Debug, Serialize, Deserialize)]
pub struct WithId<T> {
    /// id of the entry.
    pub id: Ulid,

    /// value of the entry.
    #[serde(flatten)]
    pub value: T,
}

pub(crate) fn to_bytes<T: Serialize>(value: &T) -> ron::Result<Vec<u8>> {
    let text = ron::to_string(value)?;
    Ok(text.into_bytes())
}

pub(crate) fn from_bytes<T: DeserializeOwned>(bytes: &[u8]) -> ron::Result<T> {
    let text = String::from_utf8_lossy(bytes);
    let value = ron::from_str(&text)?;
    Ok(value)
}

fn check_unique<T: Table>(db: &DB, item: &T) -> Result<()> {
    for index in T::UNIQUE_INDICES {
        let unique_val = item.unique_value(*index);
        let key = unique_key(T::TABLE, *index, &unique_val);
        let value = db.get(key)?;
        if value.is_some() {
            return Err(Error::AlreadyExists);
        }
    }

    Ok(())
}
