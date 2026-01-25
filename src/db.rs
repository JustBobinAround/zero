mod primitives;
// pub mod rand;
pub mod system_tables;
// mod uuid;
// use uuid::UUID;
//
use uuid::UUID;

use crate::{ToDatabaseBytes, db::system_tables::User, stream_writer::StreamWritable};
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
    table_version_maps: HashMap<&'static str, Vec<&'static str>>,
}

impl ToDatabaseBytes for PageMap {
    fn to_db_bytes(self) -> DatabaseBytes {
        unimplemented!()
    }

    fn from_db_bytes(bytes: &mut DatabaseBytes) -> Result<Self, ()> {
        unimplemented!()
    }
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
            table_version_maps: HashMap::new(),
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

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
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
        // impl ToDatabaseBytes for Vec<$t> {
        //     fn to_db_bytes(self) -> DatabaseBytes {
        //         let b: Vec<u8> = self
        //             .into_iter()
        //             .map(|s| s.to_le_bytes().to_vec())
        //             .flatten()
        //             .collect();

        //         DatabaseBytes::new(b.len(), b)
        //     }

        //     fn from_db_bytes(bytes: &mut DatabaseBytes) -> Result<Self, ()> {
        //         let raw = bytes.consume_layout()?;

        //         let len = raw.len() / std::mem::size_of::<$t>();
        //         let mut out = Vec::new();

        //         for i in 0..len {
        //             let mut buf = [0_u8; $bytes];
        //             let start = i * $bytes;
        //             for j in start..start + $bytes {
        //                 buf[j - start] = raw[j];
        //             }
        //             out.push(<$t>::from_le_bytes(buf));
        //         }

        //         Ok(out)
        //     }
        // }
    };
}

//TODO: u8 needs manual impl for vec
impl_to_db_bytes!(u8, 1);
impl_to_db_bytes!(u16, 2);
impl_to_db_bytes!(u32, 4);
impl_to_db_bytes!(u64, 8);
impl_to_db_bytes!(usize, 8);
impl_to_db_bytes!(u128, 16);
impl_to_db_bytes!(i8, 1);
impl_to_db_bytes!(i16, 2);
impl_to_db_bytes!(i32, 4);
impl_to_db_bytes!(i64, 8);
impl_to_db_bytes!(isize, 8);
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

// impl ToDatabaseBytes for Vec<char> {
//     fn to_db_bytes(self) -> DatabaseBytes {
//         let b: Vec<u8> = self
//             .into_iter()
//             .map(|s| (s as u8).to_le_bytes().to_vec())
//             .flatten()
//             .collect();

//         DatabaseBytes::new(b.len(), b)
//     }

//     fn from_db_bytes(bytes: &mut DatabaseBytes) -> Result<Self, ()> {
//         let raw = bytes.consume_layout()?;

//         Ok(raw.into_iter().map(|i| i as char).collect())
//     }
// }

struct DatabaseVec<T: ToDatabaseBytes> {
    t_len: usize,
    data: Vec<u8>,
    _ty: std::marker::PhantomData<T>,
}

impl<T: ToDatabaseBytes> From<Vec<T>> for DatabaseVec<Vec<T>> {
    fn from(mut value: Vec<T>) -> Self {
        if let Some(first) = value.pop() {
            let first = first.to_db_bytes().into_bytes();
            let t_len = first.len();
            let end_data: Vec<u8> = value
                .into_iter()
                .map(|b| b.to_db_bytes().into_bytes())
                .flatten()
                .collect();
            let data = [end_data, first].concat();
            DatabaseVec {
                t_len,
                data,
                _ty: std::marker::PhantomData,
            }
        } else {
            DatabaseVec {
                t_len: 0,
                data: Vec::new(),
                _ty: std::marker::PhantomData,
            }
        }
    }
}

impl<T: ToDatabaseBytes> ToDatabaseBytes for DatabaseVec<T> {
    fn to_db_bytes(self) -> DatabaseBytes {
        DatabaseBytes::default()
            .push_into(self.t_len)
            .push_db_bytes(DatabaseBytes::new(self.data.len(), self.data))
    }

    fn from_db_bytes(bytes: &mut DatabaseBytes) -> Result<Self, ()> {
        let data = bytes.consume_layout()?;
        let t_len = <usize>::from_db_bytes(bytes)?;
        Ok(DatabaseVec {
            t_len,
            data,
            _ty: std::marker::PhantomData,
        })
    }
}
impl<A: ToDatabaseBytes> ToDatabaseBytes for Vec<A> {
    fn to_db_bytes(self) -> DatabaseBytes {
        let db_vec: DatabaseVec<Vec<A>> = self.into();
        db_vec.to_db_bytes()
    }

    fn from_db_bytes(bytes: &mut DatabaseBytes) -> Result<Self, ()> {
        let db_vec = DatabaseVec::<Vec<A>>::from_db_bytes(bytes)?;

        let mut v = Vec::new();
        for chunk in db_vec.data.chunks(db_vec.t_len) {
            let mut db_bytes = DatabaseBytes::new(db_vec.t_len, chunk.to_vec());
            v.push(A::from_db_bytes(&mut db_bytes)?);
        }

        Ok(v)
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
// impl<A: ToDatabaseBytes> ToDatabaseBytes for HashMap<A, B> {
//     fn to_db_bytes(self) -> DatabaseBytes {

//     }
//     fn from_db_bytes(bytes: &mut DatabaseBytes) -> Result<Self, ()> {

//     }
// }

/// This is implemented manually to avoid circular dependency of trait and macro
impl ToDatabaseBytes for UUID {
    fn to_db_bytes(self) -> ::zero::db::DatabaseBytes {
        DatabaseBytes::default()
            .push_into(self.data_1)
            .push_into(self.data_2)
            .push_into(self.data_3)
            .push_into(self.data_4)
    }

    fn from_db_bytes(bytes: &mut DatabaseBytes) -> Result<Self, ()> {
        Ok(Self {
            data_4: <[u8; 8]>::from_db_bytes(bytes)?,
            data_3: <u16>::from_db_bytes(bytes)?,
            data_2: <u16>::from_db_bytes(bytes)?,
            data_1: <u32>::from_db_bytes(bytes)?,
        })
    }
}
// impl ToDatabaseBytes for UUID {}

pub struct TableReference<T: ZeroTable> {
    z_uuid: UUID,
    _ty: std::marker::PhantomData<T>,
}

impl<T: ZeroTable> ToDatabaseBytes for TableReference<T> {
    fn to_db_bytes(self) -> DatabaseBytes {
        self.z_uuid.to_db_bytes()
    }

    fn from_db_bytes(bytes: &mut DatabaseBytes) -> Result<Self, ()> {
        let z_uuid = UUID::from_db_bytes(bytes)?;
        Ok(TableReference {
            z_uuid,
            _ty: std::marker::PhantomData,
        })
    }
}

pub trait ZeroTable: ToDatabaseBytes {
    fn table_name() -> &'static str;
    fn table_version_hash() -> UUID;
}

impl<T: ZeroTable> ZeroTable for TableReference<T> {
    fn table_name() -> &'static str {
        T::table_name()
    }

    fn table_version_hash() -> UUID {
        T::table_version_hash()
    }
}

#[derive(ToDatabaseBytes)]
pub struct TableRecord<T: ToDatabaseBytes> {
    row: T,
    z_created_by: TableReference<User>,
    z_mod_count: u64,
    z_updated_by: TableReference<User>,
    z_updated_on: u64,
    z_uuid: UUID,
}

impl<T: ZeroTable> TableRecord<T> {
    pub fn new_system_record(row: T) -> Result<Self, ()> {
        let z_uuid = UUID::rand_v7()?;
        let z_updated_on = z_uuid.extract_timestamp();
        Ok(TableRecord {
            row,
            z_created_by: User::SYSTEM,
            z_mod_count: 0,
            z_updated_by: User::SYSTEM,
            z_updated_on,
            z_uuid,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db() {
        let test_vec = vec![1, 2, 3, 4, 5];
        let test_vec2 = test_vec.clone();
        eprintln!("{:#?}", test_vec);
        let mut bytes = test_vec2.to_db_bytes();
        eprintln!("{:#?}", bytes);
        let test_vec2 = <Vec<i32>>::from_db_bytes(&mut bytes).expect("Failed to parse db bytes");
        assert_eq!(test_vec, test_vec2);
    }
}
