//! PSD/PSB reader. Accepts the subset the writer emits.

use crate::{Document, Error, Format, Result};

#[allow(dead_code)]
struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

#[allow(dead_code)]
impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn take(&mut self, n: u64) -> Result<&'a [u8]> {
        let n = usize::try_from(n).map_err(|_| Error::Malformed("truncated"))?;
        let end = self
            .pos
            .checked_add(n)
            .ok_or(Error::Malformed("truncated"))?;
        let bytes = self
            .data
            .get(self.pos..end)
            .ok_or(Error::Malformed("truncated"))?;
        self.pos = end;
        Ok(bytes)
    }

    fn skip(&mut self, n: u64) -> Result<()> {
        self.take(n).map(|_| ())
    }

    fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    fn bytes<const N: usize>(&mut self) -> Result<[u8; N]> {
        self.take(N as u64)?
            .try_into()
            .map_err(|_| Error::Malformed("truncated"))
    }

    fn u8(&mut self) -> Result<u8> {
        let [value] = self.bytes()?;
        Ok(value)
    }

    fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_be_bytes(self.bytes()?))
    }

    fn i16(&mut self) -> Result<i16> {
        Ok(i16::from_be_bytes(self.bytes()?))
    }

    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_be_bytes(self.bytes()?))
    }

    fn i32(&mut self) -> Result<i32> {
        Ok(i32::from_be_bytes(self.bytes()?))
    }

    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_be_bytes(self.bytes()?))
    }

    fn len(&mut self, format: Format) -> Result<u64> {
        match format {
            Format::Psd => self.u32().map(u64::from),
            Format::Psb => self.u64(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub format: Format,
    pub width: u32,
    pub height: u32,
    pub channel_count: u16,
    pub depth: u16,
    pub color_mode: u16,
}

pub fn read_header(input: &[u8]) -> Result<Header> {
    let header: &[u8; 26] = input
        .get(..26)
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or(Error::Malformed("header"))?;
    if &header[..4] != b"8BPS" {
        return Err(Error::Malformed("header"));
    }
    let format = match u16::from_be_bytes([header[4], header[5]]) {
        1 => Format::Psd,
        2 => Format::Psb,
        _ => return Err(Error::Malformed("header")),
    };
    Ok(Header {
        format,
        width: u32::from_be_bytes([header[18], header[19], header[20], header[21]]),
        height: u32::from_be_bytes([header[14], header[15], header[16], header[17]]),
        channel_count: u16::from_be_bytes([header[12], header[13]]),
        depth: u16::from_be_bytes([header[22], header[23]]),
        color_mode: u16::from_be_bytes([header[24], header[25]]),
    })
}

pub fn read(_input: &[u8]) -> Result<Document> {
    Err(Error::Unsupported("not implemented"))
}
