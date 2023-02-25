use ulid::Ulid;

use crate::{Error, IndexValue, Result};

pub fn key_prefix(table: u32, index: u8) -> Vec<u8> {
    let mut prefix = table.to_be_bytes().to_vec();
    prefix.push(index);
    prefix
}

pub fn primary_key(table: u32, id: Ulid) -> Vec<u8> {
    let mut key = key_prefix(table, 0);
    key.extend_from_slice(&id.as_bytes());
    key
}

pub fn unique_key(table: u32, index: u8, value: &[u8]) -> Vec<u8> {
    let mut key = key_prefix(table, index);
    key.extend_from_slice(value);
    key
}

pub fn non_unique_prefix(table: u32, index: u8, value: &[u8]) -> Vec<u8> {
    let mut prefix = key_prefix(table, index);
    prefix.extend_from_slice(value);
    prefix
}

pub fn non_unique_key(table: u32, index: u8, value: &[u8], id: Ulid) -> Vec<u8> {
    let mut key = non_unique_prefix(table, index, value);
    key.extend_from_slice(&id.as_bytes());
    key
}

pub fn id_from_primary_key(bytes: &[u8]) -> Result<Ulid> {
    let bytes = bytes[5..].try_into().map_err(|_| Error::MalformedKey)?;
    let num = u128::from_be_bytes(bytes);
    Ok(Ulid(num))
}

pub fn id_from_non_unique_key(bytes: &[u8]) -> Result<Ulid> {
    let bytes = &bytes[bytes.len() - 16..];
    let bytes = bytes.try_into().map_err(|_| Error::MalformedKey)?;
    let num = u128::from_be_bytes(bytes);
    Ok(Ulid(num))
}

pub fn id_from_slice(bytes: &[u8]) -> Result<Ulid> {
    let bytes = bytes.try_into().map_err(|_| Error::MalformedKey)?;
    let num = u128::from_be_bytes(bytes);
    Ok(Ulid(num))
}
