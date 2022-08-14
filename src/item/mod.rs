use enum_as_inner::EnumAsInner;
use std::collections::HashMap;

#[cfg(feature = "with-serde")]
mod serde_impl;

pub type Map = HashMap<String, Item>;

#[derive(Debug, PartialEq, EnumAsInner)]
pub enum Item {
    Map(Map),
    Blob(Vec<u8>),
    List(Vec<Item>),
    Int16(i16),
    Int64(i64),
    F32(f32),
    F64(f64),
    String(String),
    Bool(bool),
    Void,
}
