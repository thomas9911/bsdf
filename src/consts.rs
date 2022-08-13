pub const PREFIX: [u8; 4] = *b"BSDF";

pub const CHECKSUM_SET: u8 = 0xFF;

pub const LARGE_SIZE: u8 = 253;
pub const SMALL_SIZE_CUTOFF: u8 = 251;

pub const COMPRESSION_NOT_SET: u8 = 0;
pub const COMPRESSION_ZLIB: u8 = 1;
pub const COMPRESSION_BZ2: u8 = 2;
