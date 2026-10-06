//! PSD/PSB reader. Accepts the subset the writer emits.

use crate::{
    Blend, Channels, Document, Error, Format, Group, Image, Layer, Mask, Node, Rect, Result, rle,
};

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
        self.data.len().saturating_sub(self.pos)
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

/// What a file says it is, before any refusal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    /// PSD or PSB.
    pub format: Format,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Channels in the merged image, as stored.
    pub channel_count: u16,
    /// Bits per channel, as stored.
    pub depth: u16,
    /// Photoshop's colour mode number, as stored. 3 is RGB.
    pub color_mode: u16,
}

/// Reads the 26-byte header only. Reports files that `read` refuses.
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
    }))
}

fn read_pascal_name(cursor: &mut Cursor<'_>) -> Result<String> {
    let length = u64::from(cursor.u8()?);
    let bytes = cursor.take(length)?;
    cursor.skip(padding4(length.wrapping_add(1)))?;
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
    let bytes = cursor.take(count.checked_mul(2).ok_or(Error::Malformed("layer name"))?)?;
    let (pairs, _) = bytes.as_chunks::<2>();
    let capacity = pairs
        .len()
        .checked_mul(3)
        .ok_or(Error::Malformed("layer name"))?;
    let mut name = String::new();
    name.try_reserve_exact(capacity)
        .map_err(|_| Error::Malformed("layer name"))?;
    let units = pairs.iter().map(|&pair| u16::from_be_bytes(pair));
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
    let pair_size: usize = if format == Format::Psd { 6 } else { 10 };
    if count
        .checked_mul(pair_size)
        .is_none_or(|bytes| bytes > cursor.remaining())
    {
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
            b"lsct" | b"lsdk" => {
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

struct DecodedLayer {
    record: RawLayer,
    pixels: Image,
}

struct LayerData {
    channels: Channels,
    layers: Vec<DecodedLayer>,
    flattened: bool,
}

fn rect_area(rect: Rect) -> Result<usize> {
    if rect.bottom < rect.top || rect.right < rect.left {
        return Err(Error::Malformed("rect"));
    }
    rect.area().ok_or(Error::Malformed("rect"))
}

fn row_count(cursor: &mut Cursor<'_>, format: Format) -> Result<u64> {
    match format {
        Format::Psd => cursor.u16().map(u64::from),
        Format::Psb => cursor.u32().map(u64::from),
    }
}

fn decode_plane(mut data: Cursor<'_>, rect: Rect, format: Format) -> Result<Vec<u8>> {
    let area = rect_area(rect)?;
    let malformed = || Error::Malformed("channel data");
    let compression = data.u16().map_err(|_| malformed())?;
    if !matches!(compression, 0 | 1) {
        return Err(Error::Unsupported("compression"));
    }
    if area == 0 {
        return if data.remaining() == 0 {
            Ok(Vec::new())
        } else {
            Err(malformed())
        };
    }
    let counts = if compression == 1 {
        data.take(count_bytes(rect, format)? as u64)
            .map_err(|_| malformed())?
    } else {
        &[]
    };
    let plane = decode_plane_data(&mut data, rect, format, compression, counts).map_err(
        |error| match error {
            Error::Malformed("truncated") => malformed(),
            other => other,
        },
    )?;
    if data.remaining() != 0 {
        return Err(malformed());
    }
    Ok(plane)
}

fn count_bytes(rect: Rect, format: Format) -> Result<usize> {
    let count_size = if format == Format::Psd { 2 } else { 4 };
    rect.height()
        .checked_mul(count_size)
        .ok_or(Error::Malformed("channel data"))
}

fn decode_plane_data(
    data: &mut Cursor<'_>,
    rect: Rect,
    format: Format,
    compression: u16,
    counts: &[u8],
) -> Result<Vec<u8>> {
    let area = rect_area(rect)?;
    let malformed = || Error::Malformed("channel data");
    let mut plane = Vec::new();
    if compression == 0 {
        let bytes = data.take(area as u64)?;
        plane.try_reserve_exact(area).map_err(|_| malformed())?;
        plane.extend_from_slice(bytes);
    } else {
        let mut sizes = Cursor::new(counts);
        let mut total = 0u64;
        while sizes.remaining() != 0 {
            total = total
                .checked_add(row_count(&mut sizes, format).map_err(|_| malformed())?)
                .ok_or_else(malformed)?;
        }
        let mut rows = Cursor::new(data.take(total)?);
        if area > rows.remaining().saturating_mul(64) {
            return Err(malformed());
        }
        plane.try_reserve_exact(area).map_err(|_| malformed())?;
        let mut sizes = Cursor::new(counts);
        for _ in 0..rect.height() {
            let count = row_count(&mut sizes, format).map_err(|_| malformed())?;
            let row = rows.take(count).map_err(|_| malformed())?;
            rle::decode_row(row, rect.width(), &mut plane).map_err(|_| malformed())?;
        }
    }
    Ok(plane)
}

fn interleave(rect: Rect, planes: [Option<Vec<u8>>; 4]) -> Result<Image> {
    let area = rect_area(rect)?;
    let [red, green, blue, alpha] = planes;
    let red = red.ok_or(Error::Malformed("channel data"))?;
    let green = green.ok_or(Error::Malformed("channel data"))?;
    let blue = blue.ok_or(Error::Malformed("channel data"))?;
    if red.len() != area
        || green.len() != area
        || blue.len() != area
        || alpha.as_ref().is_some_and(|plane| plane.len() != area)
    {
        return Err(Error::Malformed("channel data"));
    }
    let length = area.checked_mul(4).ok_or(Error::Malformed("rect"))?;
    let mut data = Vec::new();
    data.try_reserve_exact(length)
        .map_err(|_| Error::Malformed("channel data"))?;
    let alpha = alpha
        .iter()
        .flatten()
        .copied()
        .chain(std::iter::repeat(255));
    for (((&r, &g), &b), a) in red.iter().zip(&green).zip(&blue).zip(alpha) {
        data.extend_from_slice(&[r, g, b, a]);
    }
    Ok(Image { rect, data })
}

fn read_channel_data(mut records: LayerRecords<'_>, format: Format) -> Result<LayerData> {
    let mut layers = Vec::new();
    layers
        .try_reserve_exact(records.records.len())
        .map_err(|_| Error::Malformed("channel data"))?;
    for mut record in records.records {
        let mut planes: [Option<Vec<u8>>; 4] = [None, None, None, None];
        let mut mask_plane = None;
        for &(id, length) in &record.channels {
            let bytes = records.info.take(length)?;
            if id == -3 {
                continue;
            }
            let rect = if id == -2 {
                record
                    .mask
                    .as_ref()
                    .ok_or(Error::Malformed("mask block"))?
                    .rect
            } else {
                record.rect
            };
            let plane = decode_plane(Cursor::new(bytes), rect, format)?;
            match id {
                0 => planes[0] = Some(plane),
                1 => planes[1] = Some(plane),
                2 => planes[2] = Some(plane),
                -1 => planes[3] = Some(plane),
                -2 => mask_plane = Some(plane),
                _ => return Err(Error::Unsupported("channel id")),
            }
        }
        match (record.mask.as_mut(), mask_plane) {
            (Some(mask), Some(data)) => mask.data = data,
            (None, None) => {}
            _ => return Err(Error::Malformed("mask block")),
        }
        let pixels = interleave(record.rect, planes)?;
        layers.push(DecodedLayer { record, pixels });
    }
    records.info.skip(records.info.remaining() as u64)?;
    if let Some(ref mut section) = records.section
        && section.remaining() != 0
    {
        let global_mask_len = u64::from(section.u32()?);
        section.skip(global_mask_len)?;
        section.skip(section.remaining() as u64)?;
    }
    Ok(LayerData {
        channels: records.channels,
        layers,
        flattened: records.section.is_none(),
    })
}

fn build_tree(layers: Vec<DecodedLayer>) -> Result<Vec<Node>> {
    let mut root = Vec::new();
    let mut stack: Vec<Vec<Node>> = Vec::new();
    for DecodedLayer { record, pixels } in layers {
        let node = match record.section {
            3 => {
                if stack.len() == 64 {
                    return Err(Error::Malformed("nesting"));
                }
                stack
                    .try_reserve(1)
                    .map_err(|_| Error::Malformed("allocation"))?;
                stack.push(Vec::new());
                continue;
            }
            1 | 2 => {
                let children = stack.pop().ok_or(Error::Malformed("group nesting"))?;
                Node::Group(Group {
                    name: record.name,
                    visible: record.visible,
                    opacity: record.opacity,
                    blend: record.section_blend.unwrap_or(record.blend),
                    expanded: record.section == 1,
                    mask: record.mask,
                    children,
                })
            }
            _ => {
                if record.blend == Blend::PassThrough {
                    return Err(Error::Malformed("pass through on a layer"));
                }
                Node::Layer(Layer {
                    name: record.name,
                    visible: record.visible,
                    opacity: record.opacity,
                    blend: record.blend,
                    clip_to_below: record.clipped,
                    pixels,
                    mask: record.mask,
                })
            }
        };
        let nodes = stack.last_mut().unwrap_or(&mut root);
        nodes
            .try_reserve(1)
            .map_err(|_| Error::Malformed("allocation"))?;
        nodes.push(node);
    }
    if !stack.is_empty() {
        return Err(Error::Malformed("group nesting"));
    }
    Ok(root)
}

fn read_merged(cursor: &mut Cursor<'_>, header: Header, channels: Channels) -> Result<Image> {
    let compression = cursor.u16()?;
    if !matches!(compression, 0 | 1) {
        return Err(Error::Unsupported("compression"));
    }
    let rect = Rect {
        top: 0,
        left: 0,
        bottom: i32::try_from(header.height).map_err(|_| Error::Malformed("header"))?,
        right: i32::try_from(header.width).map_err(|_| Error::Malformed("header"))?,
    };
    let plane_counts = if compression == 1 {
        count_bytes(rect, header.format)?
    } else {
        0
    };
    let all_counts = plane_counts
        .checked_mul(usize::from(header.channel_count))
        .ok_or(Error::Malformed("channel data"))?;
    let mut counts = Cursor::new(cursor.take(all_counts as u64)?);
    let mut planes = [None, None, None, None];
    for slot in planes.iter_mut().take(usize::from(header.channel_count)) {
        let sizes = counts.take(plane_counts as u64)?;
        *slot = Some(decode_plane_data(
            cursor,
            rect,
            header.format,
            compression,
            sizes,
        )?);
    }
    if channels == Channels::Rgb {
        planes[3] = None;
    }
    let mut image = interleave(rect, planes)?;
    if channels == Channels::Rgba {
        for [r, g, b, a] in image.data.as_chunks_mut::<4>().0 {
            for colour in [r, g, b] {
                *colour = unmatte(*colour, *a);
            }
        }
    }
    Ok(image)
}

// Photoshop stores a transparent merged image blended over white.
#[allow(
    clippy::arithmetic_side_effects,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "u8 inputs keep every step inside i32, clamp keeps the result in u8"
)]
fn unmatte(colour: u8, alpha: u8) -> u8 {
    if alpha == 0 {
        return 0;
    }
    let (colour, alpha) = (i32::from(colour), i32::from(alpha));
    (((colour - 255 + alpha) * 255 + alpha / 2) / alpha).clamp(0, 255) as u8
}

fn padding4(length: u64) -> u64 {
    length.wrapping_neg() % 4
}

fn rect_pixels(rect: Rect) -> Option<u64> {
    let width = i64::from(rect.right)
        .checked_sub(i64::from(rect.left))?
        .max(0);
    let height = i64::from(rect.bottom)
        .checked_sub(i64::from(rect.top))?
        .max(0);
    u64::try_from(width)
        .ok()?
        .checked_mul(u64::try_from(height).ok()?)
}

// Pixel bytes of the returned document. None means past u64.
fn decoded_size(records: &LayerRecords<'_>, header: Header) -> Option<u64> {
    let merged = u64::from(header.width)
        .checked_mul(u64::from(header.height))?
        .checked_mul(4)?;
    let mut total = if records.section.is_none() {
        merged.checked_mul(2)?
    } else {
        merged
    };
    for record in &records.records {
        total = total.checked_add(rect_pixels(record.rect)?.checked_mul(4)?)?;
        if let Some(mask) = &record.mask {
            total = total.checked_add(rect_pixels(mask.rect)?)?;
        }
    }
    Some(total)
}

/// Reads a document. Same as `read_with_limit` with no cap.
pub fn read(input: &[u8]) -> Result<Document> {
    read_with_limit(input, u64::MAX)
}

/// Reads a document, refusing before any pixel allocation when its pixel buffers would exceed
/// `limit` bytes. Decoding holds one layer's planes on top of that.
pub fn read_with_limit(input: &[u8], limit: u64) -> Result<Document> {
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
    let resources = read_resources(&mut cursor)?;
    let records = read_layer_records(&mut cursor, header)?;
    let needed = decoded_size(&records, header).unwrap_or(u64::MAX);
    if needed > limit {
        return Err(Error::OverLimit { needed, limit });
    }
    let layers = read_channel_data(records, header.format)?;
    let mut tree = build_tree(layers.layers)?;
    let merged = read_merged(&mut cursor, header, layers.channels)?;
    if layers.flattened {
        let mut data = Vec::new();
        data.try_reserve_exact(merged.data.len())
            .map_err(|_| Error::Malformed("allocation"))?;
        data.extend_from_slice(&merged.data);
        tree.try_reserve(1)
            .map_err(|_| Error::Malformed("allocation"))?;
        tree.push(Node::Layer(Layer {
            name: String::from("Background"),
            visible: true,
            opacity: 255,
            blend: Blend::Normal,
            clip_to_below: false,
            pixels: Image {
                rect: merged.rect,
                data,
            },
            mask: None,
        }));
    }
    Ok(Document {
        width: header.width,
        height: header.height,
        channels: layers.channels,
        icc_profile: resources.icc_profile,
        resolution_dpi: resources.resolution_dpi,
        layers: tree,
        merged,
    })
}
