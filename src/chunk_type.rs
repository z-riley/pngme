use std::{fmt, str::FromStr};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChunkTypeError {
    #[error("chunk type must be 4 characters")]
    IncorrectLength,

    #[error("chunk type must be alphabetic")]
    NotAlpabetical,
}

/// PNG chunk types as defined by PNG Specification v1.2:
/// http://www.libpng.org/pub/png/spec/1.2/PNG-Structure.html
#[derive(Debug, PartialEq, Eq)]
pub struct ChunkType {
    /// A 4-byte chunk type code. For convenience in description and in examining PNG files, type
    /// codes are restricted to consist of uppercase and lowercase ASCII letters.
    bytes: [u8; 4],
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = ChunkTypeError;

    fn try_from(bytes: [u8; 4]) -> Result<Self, ChunkTypeError> {
        Ok(ChunkType { bytes })
    }
}

impl FromStr for ChunkType {
    type Err = ChunkTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 {
            return Err(ChunkTypeError::IncorrectLength);
        }

        for c in s.chars() {
            if !c.is_ascii_alphabetic() {
                return Err(ChunkTypeError::NotAlpabetical);
            }
        }

        return Ok(ChunkType {
            bytes: [
                s.as_bytes()[0],
                s.as_bytes()[1],
                s.as_bytes()[2],
                s.as_bytes()[3],
            ],
        });
    }
}

// The ToString trait is automatically implemented when Display is implemented
impl std::fmt::Display for ChunkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.bytes.to_vec()))
    }
}

#[allow(dead_code)]
impl ChunkType {
    pub fn bytes(&self) -> [u8; 4] {
        self.bytes
    }

    pub fn is_valid(&self) -> bool {
        self.is_reserved_bit_valid()
    }

    pub fn is_critical(&self) -> bool {
        // Bit 5 (value 32) of first byte

        const CRITICAL_CHUNK_BIT: u8 = 5;
        let mask: u8 = 1 << CRITICAL_CHUNK_BIT;

        // 0 if critical, 1 if ancillary
        if self.bytes[0] & mask == 0 {
            true
        } else {
            false
        }
    }

    pub fn is_public(&self) -> bool {
        // Bit 5 (value 32) of second byte
        const PUBLIC_CHUNK_BIT: u8 = 5;
        let mask: u8 = 1 << PUBLIC_CHUNK_BIT;

        // 0 if public, 1 if private
        if self.bytes[1] & mask == 0 {
            true
        } else {
            false
        }
    }

    pub fn is_reserved_bit_valid(&self) -> bool {
        // Bit 5 (value 32) of third byte

        const RESERVED_CHUNK_BIT: u8 = 5;
        let mask: u8 = 1 << RESERVED_CHUNK_BIT;

        // Reserved bit is valid if bit 5 of is 0 (capital letter)
        if self.bytes[2] & mask == 0 {
            true
        } else {
            false
        }
    }

    pub fn is_safe_to_copy(&self) -> bool {
        // Bit 5 of fourth byte

        const SAFE_CHUNK_BIT: u8 = 5;
        let mask: u8 = 1 << SAFE_CHUNK_BIT;

        // 0 if unsafe to copy, 1 if safe to copy
        if self.bytes[3] & mask == 0 {
            false
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
