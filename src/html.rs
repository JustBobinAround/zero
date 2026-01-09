use std::collections::HashMap;
pub type HTML = Vec<Tag>;
pub struct Tag {
    ty: TagType,
    attrs: HashMap<String, String>,
    content: Vec<Tag>,
}

pub enum TagType {
    Literal(&'static str),
    HeapText(String),
    P,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro() {}
}
