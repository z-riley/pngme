use crate::chunk_type::ChunkType;
use crc::{Crc, CRC_32_ISO_HDLC};
use std::fmt::{self};
use std::string;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChunkError {
    #[error("cannot parse less than {0} bytes")]
    InvalidLength(u64),

    #[error("supplied CRC value is incorrect: {got} (expected {expected})")]
    InvalidCrc { got: u32, expected: u32 },
}

#[derive(Debug)]
pub struct Chunk {
    /// A 4-byte unsigned integer giving the number of bytes in the chunk's data field. The length
    /// counts only the data field, not itself, the chunk type code, or the CRC. Zero is a valid length
    length: u32,

    /// A 4-byte chunk type code. For convenience in description and in examining PNG files, type
    /// codes are restricted to consist of uppercase and lowercase ASCII letters. However, encoders
    /// and decoders must treat the codes as fixed binary values, not character strings
    chunk_type: ChunkType,

    /// The data bytes appropriate to the chunk type, if any. This field can be of zero length
    chunk_data: Vec<u8>,

    /// A 4-byte CRC (Cyclic Redundancy Check) calculated on the preceding bytes in the chunk,
    /// including the chunk type code and chunk data fields, but not including the length field. The CRC is always present, even for chunks containing no data
    crc: u32,
}

impl TryFrom<&[u8]> for Chunk {
    type Error = ChunkError;

    fn try_from(value: &[u8]) -> Result<Self, ChunkError> {
        if value.len() < 12 {
            return Err(ChunkError::InvalidLength(12));
        }

        let mut iter = value.iter();

        // First 4 bytes into length
        let length_field = iter.by_ref().take(4).cloned().collect::<Vec<u8>>();
        let length = u32::from_be_bytes(length_field.try_into().expect("Invalid length"));

        // Next 4 bytes into chunk_type
        let chunk_type_field: [u8; 4] = iter
            .by_ref()
            .take(4)
            .cloned()
            .collect::<Vec<u8>>()
            .try_into()
            .expect("Invalid chunk type");
        let chunk_type = ChunkType::try_from(chunk_type_field).expect("Invalid chunk type");

        // All other bytes besides the last 4 into chunk_data
        let chunk_data: Vec<u8> = iter.by_ref().take(value.len() - 12).cloned().collect();

        // Calculate the CRC from the chunk type and data bytes
        let crc_bytes = iter.take(4).cloned().collect::<Vec<u8>>();
        let supplied_crc = u32::from_be_bytes(crc_bytes.try_into().expect("Invalid CRC"));

        // Check the supplied CRC value is correct
        let mut type_and_data_bytes =
            Vec::with_capacity(chunk_type.bytes().len() + chunk_data.len());
        type_and_data_bytes.extend_from_slice(&chunk_type.bytes());
        type_and_data_bytes.extend(&chunk_data);
        let real_crc = Crc::<u32>::new(&CRC_32_ISO_HDLC).checksum(&type_and_data_bytes);
        if supplied_crc != real_crc {
            return Err(ChunkError::InvalidCrc {
                got: supplied_crc,
                expected: real_crc,
            });
        }

        Ok(Chunk {
            length,
            chunk_type,
            chunk_data,
            crc: supplied_crc,
        })
    }
}

impl std::fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}, {}, {:?}, {})",
            self.length, self.chunk_type, self.chunk_data, self.crc
        )
    }
}

#[allow(dead_code)]
impl Chunk {
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Chunk {
        let mut type_and_data_bytes = Vec::with_capacity(chunk_type.bytes().len() + data.len());
        type_and_data_bytes.extend_from_slice(&chunk_type.bytes());
        type_and_data_bytes.extend(&data);

        Chunk {
            length: data.len() as u32,
            chunk_type,
            chunk_data: data.clone(),
            crc: Crc::<u32>::new(&CRC_32_ISO_HDLC).checksum(&type_and_data_bytes),
        }
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    pub fn data(&self) -> &[u8] {
        &self.chunk_data
    }

    pub fn crc(&self) -> u32 {
        self.crc
    }

    pub fn data_as_string(&self) -> Result<String, string::FromUtf8Error> {
        String::from_utf8(self.chunk_data.clone())
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4 + 4 + self.chunk_data.len() + 4);
        bytes.extend_from_slice(&self.length.to_be_bytes());
        bytes.extend_from_slice(&self.chunk_type.bytes());
        bytes.extend_from_slice(&self.chunk_data);
        bytes.extend_from_slice(&self.crc.to_be_bytes());
        bytes
    }
}

// #![allow(unused_variables)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!"
            .as_bytes()
            .to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
