use std::{collections::HashMap, fmt::Display, str::FromStr};

use crate::parsing::Parsable;

pub enum DataHolder {
    Primitive(String),
    Struct(HashMap<String, DataHolder>),
}

pub trait Serialize {
    fn serialize(self) -> DataHolder;
}

macro_rules! impl_primitive_serialize {
    ($t:ty) => {
        impl Serialize for $t {
            fn serialize(self) -> DataHolder {
                DataHolder::Primitive(self.to_string())
            }
        }
        impl Serialize for &$t {
            fn serialize(self) -> DataHolder {
                DataHolder::Primitive(self.to_string())
            }
        }
    };
}

impl_primitive_serialize!(bool);
impl_primitive_serialize!(char);
impl_primitive_serialize!(f32);
impl_primitive_serialize!(f64);
impl_primitive_serialize!(i8);
impl_primitive_serialize!(i16);
impl_primitive_serialize!(i32);
impl_primitive_serialize!(i64);
impl_primitive_serialize!(i128);
impl_primitive_serialize!(isize);
impl_primitive_serialize!(u8);
impl_primitive_serialize!(u16);
impl_primitive_serialize!(u32);
impl_primitive_serialize!(u64);
impl_primitive_serialize!(u128);
impl_primitive_serialize!(usize);
impl_primitive_serialize!(String);

pub trait Deserialize: Sized {
    fn deserialize(dh: DataHolder) -> Result<Self, ()>;
}

macro_rules! impl_primitive_deserialize {
    ($t:ty) => {
        impl Deserialize for $t {
            fn deserialize(dh: DataHolder) -> Result<Self, ()> {
                match dh {
                    DataHolder::Primitive(s) => match Self::from_str(&s) {
                        Ok(s) => Ok(s),
                        Err(_) => Err(()),
                    },
                    _ => Err(()),
                }
            }
        }
        impl Deserialize for HashMap<String, $t> {
            fn deserialize(dh: DataHolder) -> Result<Self, ()> {
                match dh {
                    DataHolder::Struct(map) => map
                        .into_iter()
                        .map(|(k, v)| Ok((k, <$t>::deserialize(v)?)))
                        .collect(),
                    _ => Err(()),
                }
            }
        }
    };
}

impl_primitive_deserialize!(bool);
impl_primitive_deserialize!(char);
impl_primitive_deserialize!(f32);
impl_primitive_deserialize!(f64);
impl_primitive_deserialize!(i8);
impl_primitive_deserialize!(i16);
impl_primitive_deserialize!(i32);
impl_primitive_deserialize!(i64);
impl_primitive_deserialize!(i128);
impl_primitive_deserialize!(isize);
impl_primitive_deserialize!(u8);
impl_primitive_deserialize!(u16);
impl_primitive_deserialize!(u32);
impl_primitive_deserialize!(u64);
impl_primitive_deserialize!(u128);
impl_primitive_deserialize!(usize);

impl Deserialize for String {
    fn deserialize(dh: DataHolder) -> Result<Self, ()> {
        match dh {
            DataHolder::Primitive(s) => Ok(s),
            _ => Err(()),
        }
    }
}
impl Deserialize for HashMap<String, String> {
    fn deserialize(dh: DataHolder) -> Result<Self, ()> {
        match dh {
            DataHolder::Struct(map) => map
                .into_iter()
                .map(|(k, v)| Ok((k, String::deserialize(v)?)))
                .collect(),
            _ => Err(()),
        }
    }
}

pub trait FromMap: Sized {
    fn from_map(map: HashMap<String, String>) -> Result<Self, ()>;
}

impl FromMap for HashMap<String, String> {
    fn from_map(map: HashMap<String, String>) -> Result<Self, ()> {
        Ok(map)
    }
}
