use std::{borrow::Cow, collections::BTreeSet};

/// The IndexValue trait has to be implemented for types
/// that are used as either unique or indexed field.
pub trait IndexValue {
    /// Returns a byte representiation of the given type.
    fn as_bytes(&self) -> Vec<u8>;
}

impl IndexValue for String {
    fn as_bytes(&self) -> Vec<u8> {
        self.bytes().collect()
    }
}

impl IndexValue for &str {
    fn as_bytes(&self) -> Vec<u8> {
        self.bytes().collect()
    }
}

impl IndexValue for Cow<'_, str> {
    fn as_bytes(&self) -> Vec<u8> {
        self.bytes().collect()
    }
}

impl IndexValue for ulid::Ulid {
    fn as_bytes(&self) -> Vec<u8> {
        self.0.to_be_bytes().to_vec()
    }
}

impl<T: IndexValue> IndexValue for Vec<T> {
    fn as_bytes(&self) -> Vec<u8> {
        self.iter().flat_map(|item| item.as_bytes()).collect()
    }
}

impl<T: IndexValue> IndexValue for BTreeSet<T> {
    fn as_bytes(&self) -> Vec<u8> {
        self.iter().flat_map(|item| item.as_bytes()).collect()
    }
}

macro_rules! impl_number {
    ($kind:ty) => {
        impl IndexValue for $kind {
            fn as_bytes(&self) -> Vec<u8> {
                self.to_be_bytes().to_vec()
            }
        }
    };
}

impl_number!(i8);
impl_number!(u8);
impl_number!(i16);
impl_number!(u16);
impl_number!(i32);
impl_number!(u32);
impl_number!(i64);
impl_number!(u64);
impl_number!(i128);
impl_number!(u128);
impl_number!(isize);
impl_number!(usize);
