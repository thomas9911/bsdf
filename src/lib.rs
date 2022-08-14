// https://bsdf.readthedocs.io/spec.html#minimal-implementation

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum InvalidExtension {
    #[error("zlib is not included")]
    ZlibNotCompiled,
    #[error("bz2 is not included")]
    Bz2NotCompiled,
    #[error("invalid compression setting")]
    InvalidCompressionSetting(u8),
}

impl From<InvalidExtension> for Error {
    fn from(err: InvalidExtension) -> Error {
        Error::InvalidExtension(err)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("not enough data")]
    MissingData,
    #[error("invalid header")]
    InvalidHeader,
    #[error("sudden missing data")]
    Eof,
    #[error("invalid size byte")]
    InvalidSize,
    #[error("String is not utf8")]
    InvalidUtf8,
    #[error("invalid blob hash")]
    InvalidBlobHash,
    #[error("invalid extension")]
    InvalidExtension(InvalidExtension),
    #[error("reading data from reader went wrong")]
    Reader(#[from] std::io::Error),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        use Error::*;
        match (self, other) {
            (MissingData, MissingData) => true,
            (InvalidHeader, InvalidHeader) => true,
            (Eof, Eof) => true,
            (InvalidSize, InvalidSize) => true,
            (InvalidUtf8, InvalidUtf8) => true,
            (InvalidBlobHash, InvalidBlobHash) => true,
            (InvalidExtension(e), InvalidExtension(f)) if e == f => true,
            (Reader(e), Reader(f)) if e.kind() == f.kind() => true,
            _ => false,
        }
    }
}

pub mod consts;
pub mod parser;
pub mod item;

pub use parser::Parser;
pub use item::{Item, Map};

#[test]
fn item_as_test() {
    let mut item = Item::List(vec![Item::Bool(true)]);
    let expected = Item::List(vec![Item::Bool(true), Item::Bool(true)]);

    assert_eq!(item.as_list(), Some(&vec![Item::Bool(true)]));

    if let Some(r) = item.as_list_mut() {
        r.push(Item::Bool(true))
    }

    assert_eq!(item, expected);
}
