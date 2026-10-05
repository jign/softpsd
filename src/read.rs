//! PSD/PSB reader. Accepts the subset the writer emits.

use crate::{Blend, Channels, Document, Error, Format, Mask, Rect, Result};

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

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

#[allow(dead_code)]
struct RawLayer {
    rect: Rect,
    channels: Vec<(i16, u64)>,
    blend: Blend,
    opacity: u8,
    clipped: bool,
    visible: bool,
    mask: Option<Mask>,
    name: String,
    section: u32,
    section_blend: Option<Blend>,
}

#[allow(dead_code)]
struct LayerRecords<'a> {
    channels: Channels,
    records: Vec<RawLayer>,
    info: Cursor<'a>,
    section: Option<Cursor<'a>>,
}

fn read_layer_records<'a>(cursor: &mut Cursor<'a>, header: Header) -> Result<LayerRecords<'a>> {
    let length = cursor.len(header.format)?;
    let mut section = Cursor::new(cursor.take(length)?);
    let length = if length == 0 {
        0
    } else {
        section.len(header.format)?
    };
    let mut info = Cursor::new(section.take(length)?);
    if length == 0 {
        return Ok(LayerRecords {
            channels: Channels::Rgb,
            records: Vec::new(),
            info,
            section: None,
        });
    }
    let count = info.i16()?;
    let channels = if count < 0 {
        if header.channel_count < 4 {
            return Err(Error::Malformed("layer count"));
        }
        Channels::Rgba
    } else {
        Channels::Rgb
    };
    let count = usize::from(count.unsigned_abs());
    if count > info.remaining() / 34 {
        return Err(Error::Malformed("truncated"));
    }
    let mut records = Vec::new();
    records
        .try_reserve_exact(count)
        .map_err(|_| Error::Malformed("layer record"))?;
    for _ in 0..count {
        records.push(read_record(&mut info, header.format)?);
    }
    Ok(LayerRecords {
        channels,
        records,
        info,
        section: Some(section),
    })
}

fn read_rect(cursor: &mut Cursor<'_>) -> Result<Rect> {
    Ok(Rect {
        top: cursor.i32()?,
        left: cursor.i32()?,
        bottom: cursor.i32()?,
        right: cursor.i32()?,
    })
}

fn read_mask(cursor: &mut Cursor<'_>) -> Result<Option<Mask>> {
    let length = cursor.u32()?;
    if length == 0 {
        return Ok(None);
    }
    if length < 20 {
        return Err(Error::Malformed("mask block"));
    }
    let mut data = Cursor::new(cursor.take(u64::from(length))?);
    let rect = read_rect(&mut data)?;
    let default = data.u8()?;
    if !matches!(default, 0 | 255) {
        return Err(Error::Malformed("mask block"));
    }
    let flags = data.u8()?;
    Ok(Some(Mask {
        rect,
        data: Vec::new(),
        default,
        disabled: flags & 0x02 != 0,
        inverted: flags & 0x04 != 0,
    }))
}

fn read_pascal_name(cursor: &mut Cursor<'_>) -> Result<String> {
    let length = u64::from(cursor.u8()?);
    let bytes = cursor.take(length)?;
    cursor.skip((4 - (length + 1) % 4) % 4)?;
    let mut name = String::new();
    name.try_reserve_exact(bytes.len())
        .map_err(|_| Error::Malformed("layer name"))?;
    for &byte in bytes {
        name.push(if byte.is_ascii() {
            char::from(byte)
        } else {
            '?'
        });
    }
    Ok(name)
}

fn read_unicode_name(data: &[u8]) -> Result<String> {
    let mut cursor = Cursor::new(data);
    let count = u64::from(cursor.u32()?);
    let bytes = cursor.take(count * 2)?;
    let capacity = (bytes.len() / 2)
        .checked_mul(3)
        .ok_or(Error::Malformed("layer name"))?;
    let mut name = String::new();
    name.try_reserve_exact(capacity)
        .map_err(|_| Error::Malformed("layer name"))?;
    let units = bytes
        .chunks_exact(2)
        .map(|pair| u16::from_be_bytes([pair[0], pair[1]]));
    for ch in char::decode_utf16(units) {
        name.push(ch.map_err(|_| Error::Malformed("layer name"))?);
    }
    Ok(name)
}

fn tag_length(cursor: &mut Cursor<'_>, format: Format, key: &[u8; 4]) -> Result<u64> {
    if format == Format::Psb
        && matches!(
            key,
            b"LMsk"
                | b"Lr16"
                | b"Lr32"
                | b"Layr"
                | b"Mt16"
                | b"Mt32"
                | b"Mtrn"
                | b"Alph"
                | b"FMsk"
                | b"lnk2"
                | b"FEid"
                | b"FXid"
                | b"PxSD"
                | b"cinf"
        )
    {
        cursor.u64()
    } else {
        cursor.u32().map(u64::from)
    }
}

fn refused_layer(key: &[u8; 4]) -> Option<&'static str> {
    match key {
        b"SoCo" | b"GdFl" | b"PtFl" | b"brit" | b"levl" | b"curv" | b"expA" | b"vibA" | b"hue2"
        | b"blnc" | b"blwh" | b"phfl" | b"mixr" | b"clrL" | b"nvrt" | b"post" | b"thrs"
        | b"selc" | b"grdm" => Some("adjustment or fill layer"),
        b"TySh" | b"tySh" => Some("text layer"),
        b"SoLd" | b"SoLE" | b"PlLd" => Some("smart object"),
        b"vmsk" | b"vsms" | b"vstk" | b"vogk" => Some("vector mask or shape"),
        _ => None,
    }
}

fn read_record(cursor: &mut Cursor<'_>, format: Format) -> Result<RawLayer> {
    let rect = read_rect(cursor)?;
    let count = usize::from(cursor.u16()?);
    let pair_size = if format == Format::Psd { 6 } else { 10 };
    if count > cursor.remaining() / pair_size {
        return Err(Error::Malformed("truncated"));
    }
    let mut channels = Vec::new();
    channels
        .try_reserve_exact(count)
        .map_err(|_| Error::Malformed("layer record"))?;
    for _ in 0..count {
        let id = cursor.i16()?;
        if !matches!(id, -3..=2) {
            return Err(Error::Unsupported("channel id"));
        }
        channels.push((id, cursor.len(format)?));
    }
    if cursor.take(4)? != b"8BIM" {
        return Err(Error::Malformed("layer record"));
    }
    let blend = Blend::from_key(&cursor.bytes()?).ok_or(Error::Unsupported("blend mode"))?;
    let opacity = cursor.u8()?;
    let clipped = cursor.u8()? != 0;
    let visible = cursor.u8()? & 0x02 == 0;
    cursor.skip(1)?;
    let length = u64::from(cursor.u32()?);
    let mut extra = Cursor::new(cursor.take(length)?);
    let mask = read_mask(&mut extra)?;
    let ranges_len = u64::from(extra.u32()?);
    extra.skip(ranges_len)?;
    let mut name = read_pascal_name(&mut extra)?;
    let mut section = 0;
    let mut section_blend = None;
    let mut refusal = None;
    while extra.remaining() != 0 {
        let signature = extra.bytes::<4>()?;
        if !matches!(&signature, b"8BIM" | b"8B64") {
            return Err(Error::Malformed("tagged block"));
        }
        let key = extra.bytes::<4>()?;
        let length = tag_length(&mut extra, format, &key)?;
        let data = extra.take(length)?;
        match &key {
            b"luni" => name = read_unicode_name(data)?,
            b"lsct" => {
                let mut divider = Cursor::new(data);
                section = divider.u32()?;
                section_blend = None;
                if section > 3 {
                    return Err(Error::Unsupported("section kind"));
                }
                if section == 0 {
                    refusal = refusal.or(Some("unknown section kind"));
                }
                if matches!(section, 1 | 2) && divider.remaining() != 0 {
                    if divider.take(4)? != b"8BIM" {
                        return Err(Error::Malformed("tagged block"));
                    }
                    section_blend = Some(
                        Blend::from_key(&divider.bytes()?)
                            .ok_or(Error::Unsupported("blend mode"))?,
                    );
                }
            }
            _ => refusal = refusal.or(refused_layer(&key)),
        }
    }
    if let Some(reason) = refusal {
        return Err(Error::UnsupportedLayer { name, reason });
    }
    Ok(RawLayer {
        rect,
        channels,
        blend,
        opacity,
        clipped,
        visible,
        mask,
        name,
        section,
        section_blend,
    })
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
    let _layers = read_layer_records(&mut cursor, header)?;
    Err(Error::Unsupported("not implemented"))
}
