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
    collections::{BTreeMap, HashMap, VecDeque},
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Read, Seek, Write},
    os::unix::fs::FileExt,
    path::Path,
    sync::{Arc, RwLock, atomic::AtomicBool},
};

pub type PageAddress = usize;

pub enum WalOp {
    Write,
    Commit,
    Extension(usize),
}

impl WalOp {
    pub const BIT_OFFSET: usize = 52;
    pub const MASK: usize = 0xFFF << Self::BIT_OFFSET;
    pub const WRITE: usize = 1 << Self::BIT_OFFSET;
    pub const COMMIT: usize = 2 << Self::BIT_OFFSET;

    pub fn as_page_number(&self, address: usize) -> WalPageNumber {
        let op_num = match self {
            Self::Write => Self::WRITE,
            Self::Commit => Self::COMMIT,
            Self::Extension(n) => n << Self::BIT_OFFSET,
        };

        WalPageNumber(op_num | (address >> 12))
    }
}

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WalPageNumber(usize);

impl WalPageNumber {
    pub fn split(self) -> (WalOp, PageAddress) {
        let op = match self.0 & WalOp::MASK {
            WalOp::WRITE => WalOp::Write,
            WalOp::COMMIT => WalOp::Commit,
            n => WalOp::Extension(n),
        };

        (op, self.0 << 12)
    }

    pub fn from_raw(raw: usize) -> Self {
        WalPageNumber(raw)
    }

    pub fn from_parts(op: WalOp, address: usize) -> Self {
        op.as_page_number(address)
    }
}

pub type Page = [u8; 4096];

#[derive(Debug)]
pub struct BufferedRW {
    db_file: File,
    wal_file: File,
    update_ledger: HashMap<PageAddress, Arc<Page>>,
    read_buffer: HashMap<PageAddress, Arc<Page>>,
    ledger_version: usize,
    commit: usize,
}

impl BufferedRW {
    pub const MAX_BUF: usize = 1000;
    pub fn new(path: &str) -> Result<Self, ()> {
        let path = Path::new(path);
        let wal_file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(path.with_extension("zero_wal"))
            .map_err(|_| ())?;
        wal_file.unlock().map_err(|_| ())?;

        let db_file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(path)
            .map_err(|_| ())?;
        db_file.unlock().map_err(|_| ())?;

        Ok(BufferedRW {
            db_file,
            wal_file,
            update_ledger: HashMap::new(),
            read_buffer: HashMap::new(),
            ledger_version: 0,
            commit: 0,
        })
    }

    fn wal_read<T, F: Fn(&Self) -> Result<T, ()>>(&self, f: F) -> Result<T, ()> {
        self.wal_file.lock_shared().map_err(|_| ())?;
        let t = f(self);
        self.wal_file.unlock().map_err(|_| ())?;
        Ok(t?)
    }
    fn wal_read_mut<T, F: Fn(&mut Self) -> Result<T, ()>>(&mut self, f: F) -> Result<T, ()> {
        self.wal_file.lock_shared().map_err(|_| ())?;
        let t = f(self);
        self.wal_file.unlock().map_err(|_| ())?;
        Ok(t?)
    }
    fn wal_write_mut<T, F: Fn(&mut Self) -> Result<T, ()>>(&mut self, f: F) -> Result<T, ()> {
        self.wal_file.lock().map_err(|_| ())?;
        let t = f(self);
        self.wal_file.unlock().map_err(|_| ())?;
        Ok(t?)
    }

    fn db_read_mut<T, F: Fn(&mut Self) -> Result<T, ()>>(&mut self, f: F) -> Result<T, ()> {
        self.wal_file.lock_shared().map_err(|_| ())?;
        let t = f(self);
        self.wal_file.unlock().map_err(|_| ())?;
        Ok(t?)
    }
    fn db_write_mut<T, F: Fn(&mut Self) -> Result<T, ()>>(&mut self, f: F) -> Result<T, ()> {
        self.wal_file.lock().map_err(|_| ())?;
        let t = f(self);
        self.wal_file.unlock().map_err(|_| ())?;
        Ok(t?)
    }

    fn fetch_file_version(&self) -> Result<(usize, usize), ()> {
        self.wal_read(|s| {
            let mut buf = [0_8; 8];
            let len_read = s.wal_file.read_at(&mut buf, 0).map_err(|_| ())?;
            if len_read != 8 {
                return Err(());
            }
            let commit = usize::from_le_bytes(buf);
            let len_read = s.wal_file.read_at(&mut buf, 1).map_err(|_| ())?;
            if len_read != 8 {
                return Err(());
            }
            let ledger_version = usize::from_le_bytes(buf);
            Ok((commit, ledger_version))
        })
    }

    fn sync_wal(&mut self) -> Result<(), ()> {
        self.wal_read_mut(|s| {
            let mut commit = [0_u8; 8];
            let bytes_read = s.wal_file.read_at(&mut commit, 0).map_err(|e| ())?;
            if bytes_read != 8 {
                return Err(());
            }

            let mut ledger_version = [0_u8; 8];
            let bytes_read = s.wal_file.read_at(&mut ledger_version, 8).map_err(|_| ())?;
            if bytes_read != 8 {
                return Err(());
            }

            let commit = usize::from_le_bytes(commit);
            let ledger_version = usize::from_le_bytes(ledger_version);

            if commit > s.commit {
                s.update_ledger.clear();
                s.read_buffer.clear();
                s.ledger_version = 0;
                s.wal_file
                    .seek(std::io::SeekFrom::Start(16))
                    .map_err(|_| ())?;
                while s.ledger_version < ledger_version {
                    let mut page_address = [0_u8; 8];
                    let bytes_read = s.wal_file.read(&mut page_address).map_err(|_| ())?;
                    if bytes_read != 8 {
                        return Err(());
                    }

                    let mut page = [0_u8; 4096];
                    let bytes_read = s.wal_file.read(&mut page).map_err(|_| ())?;
                    if bytes_read != 4096 {
                        return Err(());
                    }

                    let page_address = usize::from_le_bytes(page_address);
                    let page = Arc::new(page);

                    s.update_ledger.insert(page_address, page.clone());
                    s.read_buffer.insert(page_address, page);

                    s.ledger_version += 1;
                }
            } else if s.ledger_version < ledger_version {
                while s.ledger_version < ledger_version {
                    let mut page_address = [0_u8; 8];
                    let bytes_read = s.wal_file.read(&mut page_address).map_err(|_| ())?;
                    if bytes_read != 8 {
                        return Err(());
                    }

                    let mut page = [0_u8; 4096];
                    let bytes_read = s.wal_file.read(&mut page).map_err(|_| ())?;
                    if bytes_read != 4096 {
                        return Err(());
                    }

                    let page_address = usize::from_le_bytes(page_address);
                    let page = Arc::new(page);

                    s.update_ledger.insert(page_address, page.clone());
                    s.read_buffer.insert(page_address, page);

                    s.ledger_version += 1;
                }
            }

            Ok(())
        })
    }

    fn update_read_buf(&mut self, page_address: PageAddress, page: Arc<Page>) {
        match self.read_buffer.get_mut(&page_address) {
            Some(found_page) => {
                *found_page = page;
            }

            None => {
                if self.read_buffer.len() >= Self::MAX_BUF && self.read_buffer.len() > 0 {
                    let rand_key = *self
                        .read_buffer
                        .keys()
                        .next()
                        .expect("read buffer found none, this should be impossible");

                    self.read_buffer.remove(&rand_key);
                }

                self.read_buffer.insert(page_address, page);
            }
        };
    }

    pub fn read_page(&mut self, page_address: &PageAddress) -> Result<Arc<Page>, ()> {
        let page_address = (page_address >> 12) << 12;
        self.sync_wal()?;
        match self.read_buffer.get(&page_address) {
            Some(wal_page) => Ok(wal_page.clone()),
            None => self.db_read_mut(|s| {
                let mut page = [0_u8; 4096];
                match s.db_file.read_at(&mut page, page_address as u64) {
                    Ok(_) => {
                        let page = Arc::new(page);
                        s.update_read_buf(page_address, page.clone());
                        Ok(page)
                    }
                    Err(_) => Err(()),
                }
            }),
        }
    }

    pub fn write_page(&mut self, page_address: &PageAddress, page: Page) -> Result<(), ()> {
        let page_address = (page_address >> 12) << 12;
        self.wal_write_mut(|s| {
            let page = Arc::new(page);
            s.update_read_buf(page_address, page.clone());
            s.wal_file
                .write(&page_address.to_le_bytes())
                .map_err(|_| ())?;
            s.wal_file.write(&*page).map_err(|_| ())?;
            s.ledger_version += 1;
            let ledger_version = s.ledger_version.to_le_bytes();
            s.update_ledger.insert(page_address, page);
            if s.update_ledger.len() > Self::MAX_BUF {
                s.read_buffer.clear();
                s.ledger_version = 0;
                s.commit = 0;
                s.wal_file.set_len(16).map_err(|_| ())?;
                s.wal_file
                    .seek(std::io::SeekFrom::Start(0))
                    .map_err(|_| ())?;
                let commit = s.commit.to_le_bytes();
                let ledger_version = s.ledger_version.to_le_bytes();
                s.wal_file.write(&commit).map_err(|_| ())?;
                s.wal_file.write(&ledger_version).map_err(|_| ())?;

                s.flush_wal()?;
            } else {
                s.wal_file.write_at(&ledger_version, 8).map_err(|_| ())?;
            }

            Ok(())
        })
    }

    pub fn flush_wal(&mut self) -> Result<(), ()> {
        self.db_write_mut(|s| {
            let mut map = HashMap::new();
            std::mem::swap(&mut s.update_ledger, &mut map);
            for (address, page) in map {
                s.db_file.write_at(&*page, address as u64).map_err(|_| ())?;
            }

            Ok(())
        })
    }
}

// impl<F: Read + Write> BufferedRW<F> {
//     pub fn new(file: F) -> Self {
//         let reader = VecDeque::new();
//         let writer =
//     }
// }

// probably going to change this to guid index
// DB should have centeral guid index.
pub struct PageMap {
    order_map: BTreeMap<UUID, PageAddress>,
    read_map: HashMap<UUID, PageAddress>,
    open_layouts: BTreeMap<usize, PageAddress>,
    table_version_maps: HashMap<&'static str, Vec<&'static str>>,
}

impl ToDatabaseBytes for (UUID, PageAddress) {
    fn to_db_bytes(self) -> DatabaseBytes {
        DatabaseBytes::default().push_into(self.0).push_into(self.1)
    }

    fn from_db_bytes(bytes: &mut DatabaseBytes) -> Result<Self, ()> {
        let page_address = <PageAddress>::from_db_bytes(bytes)?;
        let uuid = <UUID>::from_db_bytes(bytes)?;

        Ok((uuid, page_address))
    }
}

impl ToDatabaseBytes for (usize, PageAddress) {
    fn to_db_bytes(self) -> DatabaseBytes {
        DatabaseBytes::default().push_into(self.0).push_into(self.1)
    }

    fn from_db_bytes(bytes: &mut DatabaseBytes) -> Result<Self, ()> {
        let page_address = <PageAddress>::from_db_bytes(bytes)?;
        let layout = <usize>::from_db_bytes(bytes)?;

        Ok((layout, page_address))
    }
}

impl ToDatabaseBytes for PageMap {
    fn to_db_bytes(self) -> DatabaseBytes {
        let key_vals: Vec<(UUID, PageAddress)> = self.order_map.into_iter().map(|i| i).collect();

        let open_layouts: Vec<(usize, PageAddress)> =
            self.open_layouts.into_iter().map(|i| i).collect();

        DatabaseBytes::default()
            .push_into(key_vals)
            .push_into(open_layouts)
    }

    fn from_db_bytes(bytes: &mut DatabaseBytes) -> Result<Self, ()> {
        let open_layouts: BTreeMap<usize, PageAddress> =
            <Vec<(usize, PageAddress)>>::from_db_bytes(bytes)?
                .into_iter()
                .map(|(k, v)| (k, v))
                .collect();
        let key_vals = <Vec<(UUID, PageAddress)>>::from_db_bytes(bytes)?;
        let (order_map, read_map): (BTreeMap<UUID, PageAddress>, HashMap<UUID, PageAddress>) =
            key_vals
                .into_iter()
                .map(|(k, v)| ((k.clone(), v.clone()), (k.clone(), v.clone())))
                .collect();

        Ok(PageMap {
            order_map,
            read_map,
            open_layouts,
            table_version_maps: HashMap::new(),
        })
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
