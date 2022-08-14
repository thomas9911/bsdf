
// everything is little endian

use crate::consts;
use crate::{Error, InvalidExtension, Item, Map};
use std::io::Bytes;
use std::io::Read;
use std::iter::Peekable;

pub struct Parser<'a> {
    version: Option<u16>,
    reader: Peekable<Bytes<Box<dyn Read + 'a>>>,
}

impl<'a> Parser<'a> {
    pub fn new(reader: Box<dyn Read + 'a>) -> Parser<'a> {
        Parser {
            reader: reader.bytes().peekable(),
            version: None,
        }
    }

    pub fn parse(&mut self) -> Result<Option<Item>, Error> {
        self.parse_header()?;
        Ok(self.parse_item()?)
    }

    fn parse_header(&mut self) -> Result<(), Error> {
        let first_bytes: Result<Vec<u8>, _> = self.reader.by_ref().take(6).collect();
        let buffer = first_bytes?;

        if buffer.len() != 6 {
            return Err(Error::MissingData);
        }

        if buffer[0..4] != consts::PREFIX {
            return Err(Error::InvalidHeader);
        }
        let version = u16::from_le_bytes(buffer[4..6].try_into().unwrap());
        self.version = Some(version);

        Ok(())
    }

    fn parse_item(&mut self) -> Result<Option<Item>, Error> {
        let next_byte = match self.reader.next() {
            None => return Ok(None),
            Some(Err(e)) => return Err(Error::Reader(e)),
            Some(Ok(byte)) => byte,
        };

        let item = match next_byte {
            b'v' => Item::Void,
            b'n' => Item::Bool(false),
            b'y' => Item::Bool(true),
            b'h' => self.parse_int16().map(Item::Int16)?,
            b'i' => self.parse_int64().map(Item::Int64)?,
            b'f' => self.parse_f32().map(Item::F32)?,
            b'd' => self.parse_f64().map(Item::F64)?,
            b's' => self.parse_string().map(Item::String)?,
            b'l' => self.parse_list().map(Item::List)?,
            b'm' => self.parse_map().map(Item::Map)?,
            b'b' => self.parse_blob().map(Item::Blob)?,
            _ => return Ok(None),
        };

        Ok(Some(item))
    }

    #[inline]
    fn next(&mut self) -> Result<u8, Error> {
        let byte = self.reader.next().ok_or_else(|| Error::Eof)??;
        Ok(byte)
    }

    fn parse_int16(&mut self) -> Result<i16, Error> {
        let first_byte = self.next()?;
        let second_byte = self.next()?;

        Ok(i16::from_le_bytes([first_byte, second_byte]))
    }

    fn parse_int64(&mut self) -> Result<i64, Error> {
        let byte_1 = self.next()?;
        let byte_2 = self.next()?;
        let byte_3 = self.next()?;
        let byte_4 = self.next()?;
        let byte_5 = self.next()?;
        let byte_6 = self.next()?;
        let byte_7 = self.next()?;
        let byte_8 = self.next()?;

        Ok(i64::from_le_bytes([
            byte_1, byte_2, byte_3, byte_4, byte_5, byte_6, byte_7, byte_8,
        ]))
    }

    fn parse_usize(&mut self) -> Result<usize, Error> {
        let byte_1 = self.next()?;
        let byte_2 = self.next()?;
        let byte_3 = self.next()?;
        let byte_4 = self.next()?;
        let byte_5 = self.next()?;
        let byte_6 = self.next()?;
        let byte_7 = self.next()?;
        let byte_8 = self.next()?;

        Ok(usize::from_le_bytes([
            byte_1, byte_2, byte_3, byte_4, byte_5, byte_6, byte_7, byte_8,
        ]))
    }

    fn parse_f32(&mut self) -> Result<f32, Error> {
        let byte_1 = self.next()?;
        let byte_2 = self.next()?;
        let byte_3 = self.next()?;
        let byte_4 = self.next()?;

        Ok(f32::from_le_bytes([byte_1, byte_2, byte_3, byte_4]))
    }

    fn parse_f64(&mut self) -> Result<f64, Error> {
        let byte_1 = self.next()?;
        let byte_2 = self.next()?;
        let byte_3 = self.next()?;
        let byte_4 = self.next()?;
        let byte_5 = self.next()?;
        let byte_6 = self.next()?;
        let byte_7 = self.next()?;
        let byte_8 = self.next()?;

        Ok(f64::from_le_bytes([
            byte_1, byte_2, byte_3, byte_4, byte_5, byte_6, byte_7, byte_8,
        ]))
    }

    fn parse_string(&mut self) -> Result<String, Error> {
        let length = self.parse_size()?;

        let text_data: Result<Vec<u8>, _> = self.reader.by_ref().take(length).collect();
        return String::from_utf8(text_data?).map_err(|_| Error::InvalidUtf8);
    }

    fn parse_size(&mut self) -> Result<usize, Error> {
        let first_byte = self.next()?;

        match first_byte {
            consts::LARGE_SIZE => Ok(self.parse_usize()?),
            consts::SMALL_SIZE_CUTOFF..=u8::MAX => Err(Error::InvalidSize),
            l => Ok(l as usize),
        }
    }

    fn parse_map(&mut self) -> Result<Map, Error> {
        let length = self.parse_size()?;
        let mut map = Map::with_capacity(length);

        for _ in 0..length {
            let key = self.parse_string()?;
            let item = self.parse_item()?.ok_or_else(|| Error::MissingData)?;
            map.insert(key, item);
        }

        Ok(map)
    }

    fn parse_list(&mut self) -> Result<Vec<Item>, Error> {
        let length = self.parse_size()?;
        let mut list = Vec::with_capacity(length);

        for _ in 0..length {
            list.push(self.parse_item()?.ok_or_else(|| Error::MissingData)?);
        }

        Ok(list)
    }

    fn parse_blob(&mut self) -> Result<Vec<u8>, Error> {
        let allocated_size = self.parse_size()?;
        let used_size = self.parse_size()?;
        let data_size = self.parse_size()?;
        let compressed_setting = self.next()?;
        let checksum_setting = self.next()?;
        let md5_hash = if checksum_setting == consts::CHECKSUM_SET {
            let hash: Result<Vec<u8>, _> = self.reader.by_ref().take(16).collect();
            hash?
        } else {
            Vec::new()
        };
        let byte_alignment_indicator = self.next()?;

        self.skip_bytes(byte_alignment_indicator as usize);

        let mut data = Vec::with_capacity(used_size);
        for b in self.reader.by_ref().take(used_size) {
            data.push(b?)
        }

        self.skip_bytes(allocated_size.saturating_sub(used_size) as usize);

        if !Self::check_hash(&data, &md5_hash) {
            return Err(Error::InvalidBlobHash);
        }

        let data = match compressed_setting {
            consts::COMPRESSION_NOT_SET => data,
            consts::COMPRESSION_ZLIB => Self::decompress_zlib(&data, data_size)?,
            consts::COMPRESSION_BZ2 => Self::decompress_bz2(&data, data_size)?,
            _ => {
                return Err(Error::from(InvalidExtension::InvalidCompressionSetting(
                    compressed_setting,
                )))
            }
        };

        Ok(data)
    }

    #[cfg(feature = "md5")]
    fn check_hash(data: &[u8], hash: &[u8]) -> bool {
        md5::compute(data).as_slice() == hash
    }

    #[cfg(not(feature = "md5"))]
    fn check_hash(_: &[u8], _: &[u8]) -> bool {
        true
    }

    #[cfg(feature = "zlib")]
    fn decompress_zlib(data: &[u8], size: usize) -> Result<Vec<u8>, Error> {
        let mut decompressor = flate2::read::ZlibDecoder::new(&data[..]);
        let mut buffer = Vec::with_capacity(size);
        decompressor.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    #[cfg(not(feature = "zlib"))]
    fn decompress_zlib(_: &[u8], _: usize) -> Result<Vec<u8>, Error> {
        Err(Error::from(InvalidExtension::ZlibNotCompiled))
    }

    #[cfg(feature = "bz2")]
    fn decompress_bz2(data: &[u8], size: usize) -> Result<Vec<u8>, Error> {
        let mut decompressor = bzip2::read::BzDecoder::new(&data[..]);
        let mut buffer = Vec::with_capacity(size);
        decompressor.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    #[cfg(not(feature = "bz2"))]
    fn decompress_bz2(_: &[u8], _: usize) -> Result<Vec<u8>, Error> {
        Err(Error::from(InvalidExtension::Bz2NotCompiled))
    }

    fn skip_bytes(&mut self, n: usize) {
        self.reader.by_ref().take(n).for_each(|_| {});
    }
}

#[test]
fn parses_empty() {
    let data = b"";

    let mut parser = Parser::new(Box::new(data.as_slice()));

    assert_eq!(parser.parse(), Err(Error::MissingData));
}

#[test]
fn parses_version() {
    let data = b"BSDF\x04\x02";

    let mut parser = Parser::new(Box::new(data.as_slice()));

    parser.parse().unwrap();

    assert_eq!(parser.version, Some(516));
}

#[test]
fn parse_float64() {
    // copied from python
    let data = b"BSDF\x02\x02do\x12\x83\xc0\xca!\t@";

    let mut parser = Parser::new(Box::new(data.as_slice()));

    let item = parser.parse().unwrap();

    assert_eq!(item, Some(Item::F64(3.1415)));
}

#[test]
fn parses_string() {
    // copied from python
    let data = b"BSDF\x02\x02s\xfd\xc9\x02\x00\x00\x00\x00\x00\x00\nLorem ipsum dolor sit amet, consectetur adipiscing elit. Duis id ante velit. Aenean euismod, ipsum a varius finibus, eros erat tincidunt ligula, non malesuada ex ipsum et tellus. Cras id convallis mauris, mattis porttitor nulla. In urna orci, faucibus ut consequat eleifend, vulputate ac elit. Integer gravida porta arcu, id volutpat libero lobortis at. Aenean bibendum eleifend auctor. Sed lectus purus, aliquet non purus ut, feugiat tristique leo. Praesent ut leo blandit, vulputate ex sit amet, venenatis libero. Curabitur vehicula ut enim sed posuere. Aliquam nec elit fringilla, aliquet lectus sed, suscipit quam. Vivamus malesuada ligula eu luctus finibus. Proin euismod sem sit amet eros euismod rhoncus.\n";

    let expected = Item::String(String::from("\nLorem ipsum dolor sit amet, consectetur adipiscing elit. Duis id ante velit. Aenean euismod, ipsum a varius finibus, eros erat tincidunt ligula, non malesuada ex ipsum et tellus. Cras id convallis mauris, mattis porttitor nulla. In urna orci, faucibus ut consequat eleifend, vulputate ac elit. Integer gravida porta arcu, id volutpat libero lobortis at. Aenean bibendum eleifend auctor. Sed lectus purus, aliquet non purus ut, feugiat tristique leo. Praesent ut leo blandit, vulputate ex sit amet, venenatis libero. Curabitur vehicula ut enim sed posuere. Aliquam nec elit fringilla, aliquet lectus sed, suscipit quam. Vivamus malesuada ligula eu luctus finibus. Proin euismod sem sit amet eros euismod rhoncus.\n"));

    let mut parser = Parser::new(Box::new(data.as_slice()));

    let item = parser.parse().unwrap();

    assert_eq!(item, Some(expected));
}

#[test]
fn parses_normal_map() {
    // copied from python
    let data = b"BSDF\x02\x02m\x03\x04testh\x01\x00\x05test1h\x02\x00\x05test3h\x04\x00";

    let expected = Item::Map(Map::from_iter([
        (String::from("test"), Item::Int16(1)),
        (String::from("test1"), Item::Int16(2)),
        (String::from("test3"), Item::Int16(4)),
    ]));

    let mut parser = Parser::new(Box::new(data.as_slice()));

    let item = parser.parse().unwrap();

    assert_eq!(item, Some(expected));
}

#[test]
fn parses_nested_map() {
    // copied from python
    let data = b"BSDF\x02\x02m\x02\x04testh\x01\x00\x06nestedm\x03\x06nestedy\x04listl\x03h\xff\xffni\x15\xcd[\x07\x00\x00\x00\x00\x04datas\tsome text";

    let expected = Item::Map(Map::from_iter([
        (String::from("test"), Item::Int16(1)),
        (
            String::from("nested"),
            Item::Map(Map::from_iter([
                (String::from("nested"), Item::Bool(true)),
                (
                    String::from("list"),
                    Item::List(vec![
                        Item::Int16(-1),
                        Item::Bool(false),
                        Item::Int64(123456789),
                    ]),
                ),
                (
                    String::from("data"),
                    Item::String(String::from("some text")),
                ),
            ])),
        ),
    ]));

    let mut parser = Parser::new(Box::new(data.as_slice()));

    let item = parser.parse().unwrap();

    assert_eq!(item, Some(expected));
}

#[test]
fn parser_blob() {
    // copied from python
    let data = b"BSDF\x02\x02b\n\n\n\x00\xff\x7fc\xcbm\x06yr\xc3\xf3O\tK\xb7\xe7v\xa8\x03\x00\x00\x00\x01\x02\x03\x04\x05\x06\x07\x08\t\x00";

    let expected = Item::Blob(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0]);

    let mut parser = Parser::new(Box::new(data.as_slice()));

    let item = parser.parse().unwrap();

    assert_eq!(item, Some(expected));
}

#[cfg(all(test, feature = "zlib"))]
mod zlib_test {
    use super::*;

    #[test]
    fn parser_blob_zlib() {
        // copied from python
        let data = b"BSDF\x02\x02b\xfd\x12\x00\x00\x00\x00\x00\x00\x00\xfd\x12\x00\x00\x00\x00\x00\x00\x00\xfd\n\x00\x00\x00\x00\x00\x00\x00\x01\xff\x01&\xdcT\xa3\xfcr\x7f\x1f\x14sM\xb66\x05i\x00x\xdacdbfaec\xe7\xe0d\x00\x00\x00\xdc\x00.";

        let expected = Item::Blob(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0]);

        let mut parser = Parser::new(Box::new(data.as_slice()));

        let item = parser.parse().unwrap();

        assert_eq!(item, Some(expected));
    }
}

#[cfg(all(test, feature = "bz2"))]
mod bz2_test {
    use super::*;

    #[test]
    fn parser_blob_bz2() {
        // copied from python
        let data = b"BSDF\x02\x02b\xfd-\x00\x00\x00\x00\x00\x00\x00\xfd-\x00\x00\x00\x00\x00\x00\x00\xfd\n\x00\x00\x00\x00\x00\x00\x00\x02\xff\xba9+d\xdd\x11\xba.\x1b\xa5\xddo\xde\x97l}\x00BZh91AY&SYTH\x0c\xaa\x00\x00\x00\xc0\x00\x7f\xe0 \x00\"\x01\xa6\x98@\x0c\x15^h\xe3\xe9\x8b\xb9\"\x9c(H*$\x06U\x00";

        let expected = Item::Blob(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0]);

        let mut parser = Parser::new(Box::new(data.as_slice()));

        let item = parser.parse().unwrap();

        assert_eq!(item, Some(expected));
    }
}

#[test]
fn parses_from_reader() {
    // copied from python
    let data = b"BSDF\x02\x02m\x03\x04testh\x01\x00\x05test1h\x02\x00\x05test3h\x04\x00".to_vec();
    let cursor = std::io::Cursor::new(data);

    let expected = Item::Map(Map::from_iter([
        (String::from("test"), Item::Int16(1)),
        (String::from("test1"), Item::Int16(2)),
        (String::from("test3"), Item::Int16(4)),
    ]));

    let mut parser = Parser::new(Box::new(cursor));

    let item = parser.parse().unwrap();

    assert_eq!(item, Some(expected));
}
