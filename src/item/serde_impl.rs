/// derived serde method doesn't handle the blob variant correctly, so we implement it by hand

use crate::Item;
use crate::Map;
use serde::de::{self, SeqAccess};
use serde::ser::{self, SerializeMap, SerializeSeq};
use serde::Serialize;
use serde::{Deserialize, Deserializer};
use serde_bytes::Bytes;
use std::fmt::{self, Display, Formatter};

impl<'de> Deserialize<'de> for Item {
    #[inline]
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ItemVisitor;

        impl<'de> serde::de::Visitor<'de> for ItemVisitor {
            type Value = Item;

            #[cold]
            fn expecting(&self, fmt: &mut Formatter<'_>) -> Result<(), fmt::Error> {
                "any valid BSDF value".fmt(fmt)
            }

            #[inline]
            fn visit_some<D>(self, de: D) -> Result<Item, D::Error>
            where
                D: Deserializer<'de>,
            {
                Deserialize::deserialize(de)
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Item, E> {
                Ok(Item::Void)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Item, E> {
                Ok(Item::Void)
            }

            #[inline]
            fn visit_bool<E>(self, value: bool) -> Result<Item, E> {
                Ok(Item::Bool(value))
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<Item, E> {
                Ok(Item::Int64(value))
            }

            #[inline]
            fn visit_i32<E>(self, value: i32) -> Result<Item, E> {
                Ok(Item::Int64(value.into()))
            }

            #[inline]
            fn visit_i16<E>(self, value: i16) -> Result<Item, E> {
                Ok(Item::Int16(value))
            }

            #[inline]
            fn visit_i8<E>(self, value: i8) -> Result<Item, E> {
                Ok(Item::Int16(value.into()))
            }

            #[inline]
            fn visit_u32<E>(self, value: u32) -> Result<Item, E> {
                Ok(Item::Int64(value.into()))
            }

            #[inline]
            fn visit_u16<E>(self, value: u16) -> Result<Item, E> {
                Ok(Item::Int64(value.into()))
            }

            #[inline]
            fn visit_u8<E>(self, value: u8) -> Result<Item, E> {
                Ok(Item::Int16(value.into()))
            }

            #[inline]
            fn visit_f32<E>(self, value: f32) -> Result<Item, E> {
                Ok(Item::F32(value))
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<Item, E> {
                Ok(Item::F64(value))
            }

            #[inline]
            fn visit_string<E>(self, value: String) -> Result<Item, E> {
                Ok(Item::String(String::from(value)))
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Item, E>
            where
                E: de::Error,
            {
                self.visit_string(String::from(value))
            }

            #[inline]
            fn visit_seq<V>(self, mut visitor: V) -> Result<Item, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut vec = Vec::new();
                while let Some(elem) = visitor.next_element()? {
                    vec.push(elem);
                }
                Ok(Item::List(vec))
            }

            #[inline]
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Item::Blob(v.to_owned()))
            }

            #[inline]
            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Item::Blob(v))
            }

            #[inline]
            fn visit_map<V>(self, mut visitor: V) -> Result<Item, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut map = Map::new();

                while let Some(key) = visitor.next_key()? {
                    let val = visitor.next_value()?;
                    map.insert(key, val);
                }

                Ok(Item::Map(map))
            }
        }

        de.deserialize_any(ItemVisitor)
    }
}

impl Serialize for Item {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match *self {
            Item::Void => s.serialize_unit(),
            Item::Bool(v) => s.serialize_bool(v),
            Item::Int64(n) => s.serialize_i64(n),
            Item::Int16(n) => s.serialize_i16(n),
            Item::F32(v) => s.serialize_f32(v),
            Item::F64(v) => s.serialize_f64(v),
            Item::String(ref v) => s.serialize_str(v),
            Item::Blob(ref v) => Bytes::new(&v[..]).serialize(s),
            Item::List(ref array) => {
                let mut state = s.serialize_seq(Some(array.len()))?;
                for item in array {
                    state.serialize_element(item)?;
                }
                state.end()
            }
            Item::Map(ref map) => {
                let mut state = s.serialize_map(Some(map.len()))?;
                for (ref key, ref val) in map.iter() {
                    state.serialize_entry(key, val)?;
                }
                state.end()
            }
        }
    }
}

#[test]
fn serde_ser_test() {
    use serde_value::Value;
    use std::collections::BTreeMap;

    
    let item = serde_value::to_value(Item::Bool(true)).unwrap();
    assert_eq!(Value::Bool(true), item);
    let item = serde_value::to_value(Item::Void).unwrap();
    assert_eq!(Value::Unit, item);
    let item = serde_value::to_value(Item::Int16(8)).unwrap();
    assert_eq!(Value::I16(8), item);
    let item = serde_value::to_value(Item::Blob(vec![1, 2, 3])).unwrap();
    assert_eq!(Value::Bytes(vec![1, 2, 3]), item);

    let item = serde_value::to_value(Item::String(String::from("text"))).unwrap();
    assert_eq!(Value::String(String::from("text")), item);

    let item = serde_value::to_value(Item::Map(Map::from_iter([(
        String::from("text"),
        Item::Int64(12345),
    )])))
    .unwrap();
    assert_eq!(
        Value::Map(BTreeMap::from_iter([(
            Value::String(String::from("text")),
            Value::I64(12345)
        )])),
        item
    );
}

#[test]
fn serde_de_test() {
    use serde::Deserialize;
    use serde_value::Value;
    use std::collections::BTreeMap;

    fn from_value(val: serde_value::Value) -> Result<Item, serde_value::DeserializerError> {
        Item::deserialize(serde_value::ValueDeserializer::new(val))
    }

    let item = from_value(Value::Bool(true)).unwrap();
    assert_eq!(Item::Bool(true), item);
    let item = from_value(Value::Unit).unwrap();
    assert_eq!(Item::Void, item);
    let item = from_value(Value::U8(1)).unwrap();
    assert_eq!(Item::Int16(1), item);
    let item = from_value(Value::U16(1)).unwrap();
    assert_eq!(Item::Int64(1), item);
    let item = from_value(Value::U32(1)).unwrap();
    assert_eq!(Item::Int64(1), item);
    let item = from_value(Value::U64(1));
    assert!(item.is_err());
    let item = from_value(Value::Bool(true)).unwrap();
    assert_eq!(Item::Bool(true), item);
    let item = from_value(Value::Bytes(vec![1, 2, 3])).unwrap();
    assert_eq!(Item::Blob(vec![1, 2, 3]), item);
    let item = from_value(Value::String(String::from("text"))).unwrap();
    assert_eq!(Item::String(String::from("text")), item);

    let item = from_value(Value::Map(BTreeMap::from_iter([(
        Value::String(String::from("text")),
        Value::I64(12345),
    )])))
    .unwrap();
    assert_eq!(
        Item::Map(Map::from_iter([(
            String::from("text"),
            Item::Int64(12345),
        )])),
        item
    );
}
