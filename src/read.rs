//! PSD/PSB reader. Accepts the subset the writer emits.

use crate::{Document, Error, Format, Result};

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
