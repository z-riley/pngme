use crate::chunk_type::ChunkType;
use crc::{Crc, CRC_32_ISO_HDLC};
use std::fmt::{self};
use std::string;

#[derive(Debug)]
pub struct Chunk {
    length: u32, // number of bytes in the data field
    chunk_type: ChunkType,
    chunk_data: Vec<u8>,
    crc: u32, // calculated on the preceding bytes in the chunk, including the chunk type code and chunk data fields, but not including the length field
}

impl TryFrom<&[u8]> for Chunk {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        assert!(value.len() >= 12, "Must be 12 bytes or greater");
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

        let mut chunk = Chunk {
            length: length,
            chunk_type: chunk_type,
            chunk_data: chunk_data,
            crc: 0,
        };

        // Lastly, populate the crc field using the last 4 bytes
        let crc_field = iter.take(4).cloned().collect::<Vec<u8>>();
        let supplied_crc = u32::from_be_bytes(crc_field.try_into().expect("Invalid CRC"));
        let real_crc = chunk.crc();
        if supplied_crc != real_crc {
            return Err("Supplied CRC is incorrect");
        }
        chunk.crc = supplied_crc;

        Ok(chunk)
    }
}

impl std::fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}, {}, {}, {})",
            self.length,
            self.chunk_type,
            self.data_as_string()
                .expect("Couldn't convert data to string"),
            self.crc
        )
    }
}

impl Chunk {
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Chunk {
        Chunk {
            length: data.len() as u32,
            chunk_type: chunk_type,
            chunk_data: data.clone(),
            crc: 0,
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
        let mut type_and_data_bytes =
            Vec::with_capacity(&self.chunk_type.bytes().len() + self.chunk_data.len());
        type_and_data_bytes.extend_from_slice(&self.chunk_type.bytes());
        type_and_data_bytes.extend(self.chunk_data.iter());
        Crc::<u32>::new(&CRC_32_ISO_HDLC).checksum(&type_and_data_bytes)
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
