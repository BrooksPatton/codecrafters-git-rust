// Implemented using https://dev.to/calebsander/git-internals-part-2-packfiles-1jg8 as a reference

use anyhow::Result;
use std::io::Read;

const VARINT_ENCODING_BITS: u8 = 7;
const VARINT_CONTINUE_FLAG: u8 = 1 << VARINT_ENCODING_BITS;
const TYPE_BITS: u8 = 3;
const TYPE_BYTE_SIZE_BITS: u8 = VARINT_ENCODING_BITS - TYPE_BITS;

pub fn read_varint_byte<R: Read>(packfile_reader: &mut R) -> Result<(u8, bool)> {
    let mut bytes: [u8; 1] = [0];

    packfile_reader.read_exact(&mut bytes)?;

    let [byte] = bytes;
    let value = byte & !VARINT_CONTINUE_FLAG;
    let more_bytes = byte & VARINT_CONTINUE_FLAG != 0;

    Ok((value, more_bytes))
}

pub fn read_size_encoding<R: Read>(packfile_reader: &mut R) -> Result<usize> {
    let mut value = 0;
    let mut length = 0;

    loop {
        let (byte_value, more_bytes) = read_varint_byte(packfile_reader)?;

        value |= (byte_value as usize) << length;

        if !more_bytes {
            return Ok(value);
        }

        length += VARINT_ENCODING_BITS;
    }
}

pub fn keep_bits(value: usize, bits: u8) -> usize {
    value & ((1 << bits) - 1)
}

pub fn read_type_and_size<R: Read>(packfile_reader: &mut R) -> Result<ObjectType> {
    let value = read_size_encoding(packfile_reader)?;
    let object_type = keep_bits(value >> TYPE_BYTE_SIZE_BITS, TYPE_BITS) as u8;
    let size = keep_bits(value, TYPE_BYTE_SIZE_BITS)
        | (value >> VARINT_ENCODING_BITS << TYPE_BYTE_SIZE_BITS);

    Ok(ObjectType::new(object_type, size))
}

#[derive(Debug)]
pub enum ObjectType {
    Commit(usize),
    Tree(usize),
    Blob(usize),
    Tag(usize),
    OfsDelta(usize),
    RefDelta(usize),
    Unknown,
}

impl ObjectType {
    pub fn new(object_type: u8, size: usize) -> Self {
        match object_type {
            1 => Self::Commit(size),
            2 => Self::Tree(size),
            3 => Self::Blob(size),
            4 => Self::Tag(size),
            6 => Self::OfsDelta(size),
            7 => Self::RefDelta(size),
            _ => Self::Unknown,
        }
    }

    pub fn get_size(&self) -> Option<usize> {
        Some(*match self {
            ObjectType::Commit(size) => size,
            ObjectType::Tree(size) => size,
            ObjectType::Blob(size) => size,
            ObjectType::Tag(size) => size,
            ObjectType::OfsDelta(size) => size,
            ObjectType::RefDelta(size) => size,
            ObjectType::Unknown => unreachable!(),
        })
    }

    pub fn get_type(&self) -> &'static str {
        match self {
            Self::Commit(_) => "commit",
            Self::Tree(_) => "tree",
            Self::Unknown => unreachable!(),
            ObjectType::Blob(_) => "blob",
            ObjectType::Tag(_) => "tag",
            ObjectType::OfsDelta(_) => "ofsdelta",
            ObjectType::RefDelta(_) => "refdelta",
        }
    }

    pub fn is_delta(&self) -> bool {
        matches!(self, ObjectType::OfsDelta(_) | ObjectType::RefDelta(_))
    }
}
