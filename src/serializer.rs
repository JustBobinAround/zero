use std::collections::HashMap;

pub trait FromMap: Sized {
    fn from_map(map: HashMap<String, String>) -> Result<Self, ()>;
}

impl FromMap for HashMap<String, String> {
    fn from_map(map: HashMap<String, String>) -> Result<Self, ()> {
        Ok(map)
    }
}
