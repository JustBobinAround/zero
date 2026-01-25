mod primitives;
pub mod rand;
pub mod uuid;

use crate::{ToDatabaseBytes, db::uuid::UUID, stream_writer::StreamWritable};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, HashMap},
};

pub type PageAddress = usize;

// probably going to change this to guid index
// DB should have centeral guid index.
pub struct PageMap {
    order_map: BTreeMap<UUID, PageAddress>,
    read_map: HashMap<UUID, PageAddress>,
    open_layouts: BTreeMap<usize, PageAddress>,
}

// DATA LAYOUT
//
// [min uuid pair][max uuid pair][170 UUID + PageAddress Pairs]

impl PageMap {
    pub const PAGE_SIZE: usize = 4096;

    pub fn new() -> Self {
        PageMap {
            order_map: BTreeMap::new(),
            read_map: HashMap::new(),
            open_layouts: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self) -> Result<UUID, ()> {
        let uuid = UUID::rand_v7()?;
        // self.order_map.insert(uuid.clone(), address);
        // self.read_map.insert(uuid.clone(), address);
        Ok(uuid)
    }

    pub fn get_entry(&mut self, uuid: &UUID) -> Option<&PageAddress> {
        self.read_map.get(uuid)
    }

    pub fn get_entry_bounds(&mut self, uuid: UUID) -> Option<std::ops::Range<PageAddress>> {
        let mut iter = self
            .order_map
            .range((std::ops::Bound::Included(uuid), std::ops::Bound::Unbounded));

        let a = iter.next();
        let b = iter.next();

        match (a, b) {
            (Some((_, start)), Some((_, end))) => Some(*start..*end),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DatabaseBytes {
    layouts: Vec<usize>,
    bytes: Vec<u8>,
}

impl DatabaseBytes {
    pub fn new(layout: usize, bytes: Vec<u8>) -> Self {
        Self {
            layouts: vec![layout],
            bytes,
        }
    }

    pub fn push_into(self, other: impl ToDatabaseBytes) -> Self {
        let other = other.to_db_bytes();
        self.push_db_bytes(other)
    }

    pub fn push_db_bytes(mut self, other: Self) -> Self {
        self.layouts = [self.layouts, other.layouts].concat();
        self.bytes = [self.bytes, other.bytes].concat();

        self
    }

    pub fn consume_layout(&mut self) -> Result<Vec<u8>, ()> {
        match self.layouts.pop() {
            Some(size) if self.bytes.len() >= size => {
                Ok(self.bytes.drain(self.bytes.len() - size..).collect())
            }
            _ => Err(()),
        }
    }
}

impl Default for DatabaseBytes {
    fn default() -> Self {
        DatabaseBytes {
            layouts: Vec::new(),
            bytes: Vec::new(),
        }
    }
}

pub trait ToDatabaseBytes: Sized {
    fn to_db_bytes(self) -> DatabaseBytes;
    fn from_db_bytes(bytes: &mut DatabaseBytes) -> Result<Self, ()>;
}

macro_rules! impl_to_db_bytes {
    ($t: ty, $bytes: literal) => {
        impl ToDatabaseBytes for $t {
            fn to_db_bytes(self) -> DatabaseBytes {
                let b = self.to_le_bytes().to_vec();
                DatabaseBytes::new(b.len(), b)
            }

            fn from_db_bytes(bytes: &mut DatabaseBytes) -> Result<Self, ()> {
                let bytes = bytes.consume_layout()?;
                match bytes.split_first_chunk::<$bytes>() {
                    Some((b, _)) => Ok(<$t>::from_le_bytes(*b)),
                    _ => Err(()),
                }
            }
        }
        impl<const N: usize> ToDatabaseBytes for [$t; N] {
            fn to_db_bytes(self) -> DatabaseBytes {
                let b: Vec<u8> = self
                    .into_iter()
                    .map(|s| s.to_le_bytes().to_vec())
                    .flatten()
                    .collect();

                DatabaseBytes::new(b.len(), b)
            }

            fn from_db_bytes(bytes: &mut DatabaseBytes) -> Result<Self, ()> {
                let raw = bytes.consume_layout()?;

                if raw.len() != N * $bytes {
                    return Err(());
                }

                let mut out: [$t; N] = [0; N];
                for i in 0..N {
                    let mut buf = [0_u8; $bytes];
                    let start = i * $bytes;
                    for j in start..start + $bytes {
                        buf[j - start] = raw[j];
                    }
                    out[i] = <$t>::from_le_bytes(buf);
                }

                Ok(out)
            }
        }
        impl ToDatabaseBytes for Vec<$t> {
            fn to_db_bytes(self) -> DatabaseBytes {
                let b: Vec<u8> = self
                    .into_iter()
                    .map(|s| s.to_le_bytes().to_vec())
                    .flatten()
                    .collect();

                DatabaseBytes::new(b.len(), b)
            }

            fn from_db_bytes(bytes: &mut DatabaseBytes) -> Result<Self, ()> {
                let raw = bytes.consume_layout()?;

                let len = raw.len() / std::mem::size_of::<$t>();
                let mut out = Vec::new();

                for i in 0..len {
                    let mut buf = [0_u8; $bytes];
                    let start = i * $bytes;
                    for j in start..start + $bytes {
                        buf[j - start] = raw[j];
                    }
                    out.push(<$t>::from_le_bytes(buf));
                }

                Ok(out)
            }
        }
    };
}

impl_to_db_bytes!(u8, 1);
impl_to_db_bytes!(u16, 2);
impl_to_db_bytes!(u32, 4);
impl_to_db_bytes!(u64, 8);
impl_to_db_bytes!(u128, 16);
impl_to_db_bytes!(i8, 1);
impl_to_db_bytes!(i16, 2);
impl_to_db_bytes!(i32, 4);
impl_to_db_bytes!(i64, 8);
impl_to_db_bytes!(i128, 16);

impl ToDatabaseBytes for char {
    fn to_db_bytes(self) -> DatabaseBytes {
        let b = (self as u8).to_le_bytes().to_vec();
        DatabaseBytes::new(b.len(), b)
    }

    fn from_db_bytes(bytes: &mut DatabaseBytes) -> Result<Self, ()> {
        let bytes = bytes.consume_layout()?;
        match bytes.split_first_chunk::<1>() {
            Some((b, _)) => Ok(<u8>::from_le_bytes(*b) as char),
            _ => Err(()),
        }
    }
}

impl<const N: usize> ToDatabaseBytes for [char; N] {
    fn to_db_bytes(self) -> DatabaseBytes {
        let b: Vec<u8> = self
            .into_iter()
            .map(|s| (s as u8).to_le_bytes().to_vec())
            .flatten()
            .collect();

        DatabaseBytes::new(b.len(), b)
    }

    fn from_db_bytes(bytes: &mut DatabaseBytes) -> Result<Self, ()> {
        let raw = bytes.consume_layout()?;

        if raw.len() != N {
            return Err(());
        }

        let mut out: [char; N] = ['\0'; N];
        for i in 0..N {
            out[i] = raw[i] as char;
        }

        Ok(out)
    }
}

impl ToDatabaseBytes for Vec<char> {
    fn to_db_bytes(self) -> DatabaseBytes {
        let b: Vec<u8> = self
            .into_iter()
            .map(|s| (s as u8).to_le_bytes().to_vec())
            .flatten()
            .collect();

        DatabaseBytes::new(b.len(), b)
    }

    fn from_db_bytes(bytes: &mut DatabaseBytes) -> Result<Self, ()> {
        let raw = bytes.consume_layout()?;

        Ok(raw.into_iter().map(|i| i as char).collect())
    }
}

impl ToDatabaseBytes for String {
    fn to_db_bytes(self) -> DatabaseBytes {
        let b = self.into_bytes();

        DatabaseBytes::new(b.len(), b)
    }

    fn from_db_bytes(bytes: &mut DatabaseBytes) -> Result<Self, ()> {
        let bytes = bytes.consume_layout()?;
        let s = String::from_utf8(bytes).map_err(|_| ())?;

        Ok(s)
    }
}

impl<T: ToDatabaseBytes> ToDatabaseBytes for Option<T> {
    fn to_db_bytes(self) -> DatabaseBytes {
        match self {
            Some(t) => t.to_db_bytes(),
            None => DatabaseBytes::new(0, vec![]),
        }
    }

    fn from_db_bytes(bytes: &mut DatabaseBytes) -> Result<Self, ()> {
        let bytes = bytes.consume_layout()?;
        if bytes.len() == 0 {
            Ok(None)
        } else {
            let mut bytes = DatabaseBytes::new(bytes.len(), bytes);
            Ok(Some(T::from_db_bytes(&mut bytes)?))
        }
    }
}

#[derive(ToDatabaseBytes)]
pub struct TableRecord<T: ToDatabaseBytes> {
    z_uuid: UUID,
    z_created_by: UUID,
    z_updated_on: u64,
    z_updated_by: UUID,
    data: T,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db() {
        // let a = TestT { a: 12, b: 13 };
        // let mut db = a.to_db_bytes();
        // let db = TestT::from_db_bytes(&mut db);
        // assert_eq!(Ok(TestT { a: 12, b: 13 }), db);
    }
}
