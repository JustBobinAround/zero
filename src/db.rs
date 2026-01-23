mod primitives;
use std::collections::BTreeMap;

// probably going to change this to guid index
// DB should have centeral guid index.
pub struct Table {
    rows: BTreeMap<usize, DatabaseBytes>,
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
            Some(size) if self.bytes.len() > size => {
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

// This will all be macro built, just testing
impl ToDatabaseBytes for u8 {
    fn to_db_bytes(self) -> DatabaseBytes {
        let b = self.to_le_bytes().to_vec();

        DatabaseBytes::new(b.len(), b)
    }

    fn from_db_bytes(bytes: &mut DatabaseBytes) -> Result<Self, ()> {
        let bytes = bytes.consume_layout()?;
        match bytes.split_first_chunk::<1>() {
            Some((b, _)) => Ok(u8::from_le_bytes(*b)),
            _ => Err(()),
        }
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
