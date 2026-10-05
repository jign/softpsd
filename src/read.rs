//! PSD/PSB reader. Accepts the subset the writer emits.

use crate::{Document, Error, Format, Result};

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

#[allow(dead_code)]
struct Resources {
    icc_profile: Option<Vec<u8>>,
    resolution_dpi: Option<f32>,
}

fn read_resources(cursor: &mut Cursor<'_>) -> Result<Resources> {
    let length = u64::from(cursor.u32()?);
    let mut blocks = Cursor::new(cursor.take(length)?);
    let mut resources = Resources {
        icc_profile: None,
        resolution_dpi: None,
    };
    while blocks.remaining() != 0 {
        if blocks.take(4)? != b"8BIM" {
            return Err(Error::Malformed("image resources"));
        }
        let id = blocks.u16()?;
        let name_len = u64::from(blocks.u8()?);
        blocks.skip(name_len)?;
        blocks.skip((name_len + 1) % 2)?;
        let data_len = u64::from(blocks.u32()?);
        let data = blocks.take(data_len)?;
        blocks.skip(data_len % 2)?;
        match id {
            1039 => {
                let mut profile = Vec::new();
                profile
                    .try_reserve_exact(data.len())
                    .map_err(|_| Error::Malformed("image resources"))?;
                profile.extend_from_slice(data);
                resources.icc_profile = Some(profile);
            }
            1005 => {
                let mut resolution = Cursor::new(data);
                let value = resolution.u32()? as f32 / 65_536.0;
                match resolution.u16()? {
                    1 => resources.resolution_dpi = Some(value),
                    2 => resources.resolution_dpi = Some(value * 2.54),
                    _ => {}
                }
            }
            _ => {}
        }
    }
    Ok(resources)
}

pub fn read(input: &[u8]) -> Result<Document> {
    let header = read_header(input)?;
    if header.depth != 8 {
        return Err(Error::Unsupported("depth"));
    }
    if header.color_mode != 3 {
        return Err(Error::Unsupported("colour mode"));
    }
    if !(3..=4).contains(&header.channel_count) {
        return Err(Error::Unsupported("channel count"));
    }
    let side_limit = match header.format {
        Format::Psd => 30_000,
        Format::Psb => 300_000,
    };
    if !(1..=side_limit).contains(&header.width) || !(1..=side_limit).contains(&header.height) {
        return Err(Error::Malformed("header"));
    }
    let mut cursor = Cursor::new(input);
    cursor.skip(26)?;
    let colour_len = u64::from(cursor.u32()?);
    cursor.skip(colour_len)?;
    let _resources = read_resources(&mut cursor)?;
    Err(Error::Unsupported("not implemented"))
}
