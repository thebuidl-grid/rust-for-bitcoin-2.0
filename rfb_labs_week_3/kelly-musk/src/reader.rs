//! A tiny cursor over a byte slice. Bitcoin serialization is little-endian for
//! integers and uses the CompactSize (a.k.a. VarInt) encoding for lengths.

use crate::error::{ParseError, Result};

pub struct ByteReader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> ByteReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn total(&self) -> usize {
        self.bytes.len()
    }

    pub fn remaining(&self) -> usize {
        self.bytes.len() - self.pos
    }

    pub fn read_bytes(&mut self, len: usize, what: &'static str) -> Result<&'a [u8]> {
        let end = self
            .pos
            .checked_add(len)
            .filter(|end| *end <= self.bytes.len())
            .ok_or(ParseError::UnexpectedEof {
                offset: self.pos,
                what,
            })?;
        let slice = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    pub fn read_array<const N: usize>(&mut self, what: &'static str) -> Result<[u8; N]> {
        let slice = self.read_bytes(N, what)?;
        let mut out = [0_u8; N];
        out.copy_from_slice(slice);
        Ok(out)
    }

    pub fn read_u8(&mut self, what: &'static str) -> Result<u8> {
        Ok(self.read_array::<1>(what)?[0])
    }

    pub fn read_u16_le(&mut self, what: &'static str) -> Result<u16> {
        Ok(u16::from_le_bytes(self.read_array::<2>(what)?))
    }

    pub fn read_u32_le(&mut self, what: &'static str) -> Result<u32> {
        Ok(u32::from_le_bytes(self.read_array::<4>(what)?))
    }

    pub fn read_u64_le(&mut self, what: &'static str) -> Result<u64> {
        Ok(u64::from_le_bytes(self.read_array::<8>(what)?))
    }

    /// Read a CompactSize integer and reject over-long (non-minimal) encodings,
    /// which is what Bitcoin Core's `ReadCompactSize` does with `fRejectNegative`.
    pub fn read_compact_size(&mut self, what: &'static str) -> Result<u64> {
        let start = self.pos;
        let first = self.read_u8(what)?;
        let value = match first {
            0..=0xfc => u64::from(first),
            0xfd => {
                let v = u64::from(self.read_u16_le(what)?);
                if v < 0xfd {
                    return Err(ParseError::NonMinimalCompactSize { offset: start });
                }
                v
            }
            0xfe => {
                let v = u64::from(self.read_u32_le(what)?);
                if v <= 0xffff {
                    return Err(ParseError::NonMinimalCompactSize { offset: start });
                }
                v
            }
            0xff => {
                let v = self.read_u64_le(what)?;
                if v <= 0xffff_ffff {
                    return Err(ParseError::NonMinimalCompactSize { offset: start });
                }
                v
            }
        };
        Ok(value)
    }
}

/// Encode a value as a minimal CompactSize integer (used by the serializer).
pub fn write_compact_size(out: &mut Vec<u8>, value: u64) {
    match value {
        0..=0xfc => out.push(value as u8),
        0xfd..=0xffff => {
            out.push(0xfd);
            out.extend_from_slice(&(value as u16).to_le_bytes());
        }
        0x1_0000..=0xffff_ffff => {
            out.push(0xfe);
            out.extend_from_slice(&(value as u32).to_le_bytes());
        }
        _ => {
            out.push(0xff);
            out.extend_from_slice(&value.to_le_bytes());
        }
    }
}
