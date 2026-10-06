//! PSD/PSB writer.

use crate::{
    Blend, Channels, Document, Error, Format, Group, Layer, Mask, Node, Rect, Result, rle, validate,
};
use std::io::{Seek, SeekFrom, Write};

enum Record<'a> {
    Layer(&'a Layer),
    GroupEnd,
    GroupStart(&'a Group),
}

fn flatten<'a>(nodes: &'a [Node], out: &mut Vec<Record<'a>>) {
    for node in nodes {
        match node {
            Node::Layer(layer) => out.push(Record::Layer(layer)),
            Node::Group(group) => {
                out.push(Record::GroupEnd);
                flatten(&group.children, out);
                out.push(Record::GroupStart(group));
            }
        }
    }
}

fn record_mask<'a>(record: &Record<'a>) -> Option<&'a Mask> {
    match record {
        Record::Layer(layer) => layer.mask.as_ref(),
        Record::GroupStart(group) => group.mask.as_ref(),
        Record::GroupEnd => None,
    }
}

fn channel_ids(record: &Record<'_>) -> Vec<i16> {
    let mut ids = vec![-1, 0, 1, 2];
    if record_mask(record).is_some() {
        ids.push(-2);
    }
    ids
}

// Writes one channel and returns its length. Empty layers and groups store it raw with zero rows.
fn write_channel<W: Write + Seek>(
    out: &mut W,
    record: &Record<'_>,
    id: i16,
    format: Format,
) -> Result<u64> {
    if id == -2 {
        let mask = record_mask(record).ok_or(Error::Malformed("mask"))?;
        return write_plane(out, &mask.data, mask.rect, format, |&value| value);
    }
    match record {
        Record::Layer(layer) if !layer.pixels.rect.is_empty() => {
            let (pixels, _) = layer.pixels.data.as_chunks::<4>();
            let rect = layer.pixels.rect;
            match id {
                -1 => write_plane(out, pixels, rect, format, |[.., a]| *a),
                0 => write_plane(out, pixels, rect, format, |[r, ..]| *r),
                1 => write_plane(out, pixels, rect, format, |[_, g, ..]| *g),
                _ => write_plane(out, pixels, rect, format, |[_, _, b, _]| *b),
            }
        }
        _ => {
            out.write_all(&0u16.to_be_bytes())?;
            Ok(2)
        }
    }
}

fn count_size(format: Format) -> usize {
    match format {
        Format::Psd => 2,
        Format::Psb => 4,
    }
}

fn zeros(out: &mut impl Write, count: usize) -> Result<()> {
    let count = u64::try_from(count).map_err(|_| Error::Unsupported("section length"))?;
    std::io::copy(&mut std::io::Read::take(std::io::repeat(0), count), out)?;
    Ok(())
}

// Compression, row counts, rows. The counts are written last, over a zeroed placeholder.
fn write_plane<W: Write + Seek, T>(
    out: &mut W,
    data: &[T],
    rect: Rect,
    format: Format,
    value: impl Fn(&T) -> u8,
) -> Result<u64> {
    out.write_all(&1u16.to_be_bytes())?;
    let counts_at = out.stream_position()?;
    let counts_len = rect
        .height()
        .checked_mul(count_size(format))
        .ok_or(Error::Unsupported("section length"))?;
    zeros(out, counts_len)?;
    let counts = write_rows(out, data, rect, format, value)?;
    let end = out.stream_position()?;
    out.seek(SeekFrom::Start(counts_at))?;
    out.write_all(&counts)?;
    out.seek(SeekFrom::Start(end))?;
    end.checked_sub(counts_at)
        .and_then(|len| len.checked_add(2))
        .ok_or(Error::Malformed("stream position"))
}

// Encodes and writes one row at a time; returns the row byte counts.
fn write_rows<T>(
    out: &mut impl Write,
    data: &[T],
    rect: Rect,
    format: Format,
    value: impl Fn(&T) -> u8,
) -> Result<Vec<u8>> {
    let width = rect.width();
    let height = rect.height();
    let mut counts = Vec::new();
    counts
        .try_reserve_exact(height.saturating_mul(count_size(format)))
        .map_err(|_| Error::Unsupported("allocation"))?;
    let mut row = Vec::with_capacity(width);
    let mut encoded = Vec::new();
    let mut rest = data;
    for _ in 0..height {
        let (line, tail) = rest
            .split_at_checked(width)
            .ok_or(Error::Malformed("plane size"))?;
        rest = tail;
        row.clear();
        row.extend(line.iter().map(&value));
        encoded.clear();
        rle::encode_row(&row, &mut encoded);
        match format {
            Format::Psd => {
                let count = u16::try_from(encoded.len())
                    .map_err(|_| Error::Unsupported("RLE row length"))?;
                counts.extend_from_slice(&count.to_be_bytes());
            }
            Format::Psb => {
                let count = u32::try_from(encoded.len())
                    .map_err(|_| Error::Unsupported("RLE row length"))?;
                counts.extend_from_slice(&count.to_be_bytes());
            }
        }
        out.write_all(&encoded)?;
    }
    Ok(counts)
}

/// Writes `doc` to `out`. Each channel is written as soon as it is encoded; the lengths that
/// precede the channels are filled in afterwards, which is why `out` must seek. On error, `out`
/// holds a partial file.
pub fn write<W: Write + Seek>(doc: &Document, format: Format, out: &mut W) -> Result<()> {
    validate::validate(doc, format)?;
    let mut records = Vec::new();
    flatten(&doc.layers, &mut records);
    let count = i16::try_from(records.len()).map_err(|_| Error::Unsupported("layer count"))?;
    let resources = image_resources(doc)?;
    let resources_len = length(resources.len())?;
    let version: u16 = if format == Format::Psd { 1 } else { 2 };
    let channel_count: u16 = if doc.channels == Channels::Rgb { 3 } else { 4 };
    let count = if doc.channels == Channels::Rgba {
        count
            .checked_neg()
            .ok_or(Error::Unsupported("layer count"))?
    } else {
        count
    };

    out.write_all(b"8BPS")?;
    out.write_all(&version.to_be_bytes())?;
    out.write_all(&[0; 6])?;
    out.write_all(&channel_count.to_be_bytes())?;
    out.write_all(&doc.height.to_be_bytes())?;
    out.write_all(&doc.width.to_be_bytes())?;
    out.write_all(&8u16.to_be_bytes())?;
    out.write_all(&3u16.to_be_bytes())?;
    out.write_all(&0u32.to_be_bytes())?;
    out.write_all(&resources_len.to_be_bytes())?;
    out.write_all(&resources)?;

    let section_lengths_at = out.stream_position()?;
    write_length(out, 0, format)?;
    write_length(out, 0, format)?;
    let info_start = out.stream_position()?;
    out.write_all(&count.to_be_bytes())?;
    let mut block = Vec::new();
    let mut length_fields = Vec::new();
    for (id, record) in (1u32..).zip(&records) {
        encode_record(
            record,
            &channel_ids(record),
            id,
            format,
            &mut block,
            &mut length_fields,
        )?;
    }
    let block_at = out.stream_position()?;
    out.write_all(&block)?;
    let mut channel_lengths = Vec::with_capacity(length_fields.len());
    for record in &records {
        for id in channel_ids(record) {
            channel_lengths.push(write_channel(out, record, id, format)?);
        }
    }
    let info_len = out
        .stream_position()?
        .checked_sub(info_start)
        .ok_or(Error::Malformed("stream position"))?;
    let padding = info_len.wrapping_neg() % 4;
    out.write_all(
        [0; 3]
            .get(..usize::try_from(padding).unwrap_or_default())
            .unwrap_or_default(),
    )?;
    out.write_all(&0u32.to_be_bytes())?;
    let layer_info_len = info_len
        .checked_add(padding)
        .ok_or(Error::Unsupported("section length"))?;
    let layer_mask_len = layer_info_len
        .checked_add(if format == Format::Psd { 8 } else { 12 })
        .ok_or(Error::Unsupported("section length"))?;

    out.write_all(&1u16.to_be_bytes())?;
    let counts_at = out.stream_position()?;
    let all_counts = doc
        .merged
        .rect
        .height()
        .checked_mul(count_size(format))
        .and_then(|len| len.checked_mul(usize::from(channel_count)))
        .ok_or(Error::Unsupported("section length"))?;
    zeros(out, all_counts)?;
    let (pixels, _) = doc.merged.data.as_chunks::<4>();
    let rect = doc.merged.rect;
    let mut counts = Vec::new();
    for index in 0..channel_count {
        let channel = match doc.channels {
            Channels::Rgb => match index {
                0 => write_rows(out, pixels, rect, format, |[r, ..]| *r),
                1 => write_rows(out, pixels, rect, format, |[_, g, ..]| *g),
                _ => write_rows(out, pixels, rect, format, |[_, _, b, _]| *b),
            },
            Channels::Rgba => match index {
                0 => write_rows(out, pixels, rect, format, |[r, .., a]| over_white(*r, *a)),
                1 => write_rows(out, pixels, rect, format, |[_, g, _, a]| over_white(*g, *a)),
                2 => write_rows(out, pixels, rect, format, |[_, _, b, a]| over_white(*b, *a)),
                _ => write_rows(out, pixels, rect, format, |[.., a]| *a),
            },
        }?;
        counts.extend_from_slice(&channel);
    }
    let end = out.stream_position()?;

    for (&at, &channel_length) in length_fields.iter().zip(&channel_lengths) {
        let mut value = Vec::new();
        write_length(&mut value, channel_length, format)?;
        at.checked_add(value.len())
            .and_then(|field_end| block.get_mut(at..field_end))
            .ok_or(Error::Malformed("record block"))?
            .copy_from_slice(&value);
    }
    out.seek(SeekFrom::Start(block_at))?;
    out.write_all(&block)?;
    out.seek(SeekFrom::Start(section_lengths_at))?;
    write_length(out, layer_mask_len, format)?;
    write_length(out, layer_info_len, format)?;
    out.seek(SeekFrom::Start(counts_at))?;
    out.write_all(&counts)?;
    out.seek(SeekFrom::Start(end))?;
    Ok(())
}

/// PSD up to 30,000 px per side, PSB above.
pub fn format_for(width: u32, height: u32) -> Format {
    if width > 30_000 || height > 30_000 {
        Format::Psb
    } else {
        Format::Psd
    }
}

fn length(value: usize) -> Result<u32> {
    u32::try_from(value).map_err(|_| Error::Unsupported("section length"))
}

fn write_length(out: &mut impl Write, value: u64, format: Format) -> Result<()> {
    match format {
        Format::Psd => out.write_all(
            &u32::try_from(value)
                .map_err(|_| Error::Unsupported("section length"))?
                .to_be_bytes(),
        )?,
        Format::Psb => out.write_all(&value.to_be_bytes())?,
    }
    Ok(())
}

fn rect_bytes(rect: Rect, out: &mut Vec<u8>) {
    for value in [rect.top, rect.left, rect.bottom, rect.right] {
        out.extend_from_slice(&value.to_be_bytes());
    }
}

fn pad(out: &mut Vec<u8>, alignment: usize) {
    while !out.len().is_multiple_of(alignment) {
        out.push(0);
    }
}

fn unicode(name: &str) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    data.extend_from_slice(&length(name.encode_utf16().count())?.to_be_bytes());
    for unit in name.encode_utf16() {
        data.extend_from_slice(&unit.to_be_bytes());
    }
    Ok(data)
}

fn tagged(out: &mut Vec<u8>, key: &[u8; 4], data: &[u8]) -> Result<()> {
    out.extend_from_slice(b"8BIM");
    out.extend_from_slice(key);
    out.extend_from_slice(&length(data.len())?.to_be_bytes());
    out.extend_from_slice(data);
    Ok(())
}

fn resource(out: &mut Vec<u8>, id: u16, data: &[u8]) -> Result<()> {
    out.extend_from_slice(b"8BIM");
    out.extend_from_slice(&id.to_be_bytes());
    out.extend_from_slice(&[0, 0]);
    out.extend_from_slice(&length(data.len())?.to_be_bytes());
    out.extend_from_slice(data);
    pad(out, 2);
    Ok(())
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "validate bounds it to 1..=u32::MAX"
)]
fn fixed_dpi(dpi: f32) -> u32 {
    (f64::from(dpi) * 65_536.0).round() as u32
}

fn image_resources(doc: &Document) -> Result<Vec<u8>> {
    let mut resources = Vec::new();
    if let Some(dpi) = doc.resolution_dpi {
        let fixed = fixed_dpi(dpi);
        let mut resolution = Vec::new();
        for _ in 0..2 {
            resolution.extend_from_slice(&fixed.to_be_bytes());
            resolution.extend_from_slice(&1u16.to_be_bytes());
            resolution.extend_from_slice(&1u16.to_be_bytes());
        }
        resource(&mut resources, 1005, &resolution)?;
    }
    if let Some(profile) = &doc.icc_profile {
        resource(&mut resources, 1039, profile)?;
    }
    let mut data = Vec::new();
    data.extend_from_slice(&1u32.to_be_bytes());
    data.push(1);
    data.extend_from_slice(&unicode("softpsd")?);
    data.extend_from_slice(&unicode("softpsd")?);
    data.extend_from_slice(&1u32.to_be_bytes());
    resource(&mut resources, 1057, &data)?;
    Ok(resources)
}

// Appends the record to `bytes` with zero channel lengths and pushes each length's offset.
fn encode_record(
    record: &Record<'_>,
    channel_ids: &[i16],
    id: u32,
    format: Format,
    bytes: &mut Vec<u8>,
    length_fields: &mut Vec<usize>,
) -> Result<()> {
    let empty_rect = Rect {
        top: 0,
        left: 0,
        bottom: 0,
        right: 0,
    };
    let (rect, name, blend, opacity, visible, clipped, mask) = match record {
        Record::Layer(layer) => (
            layer.pixels.rect,
            layer.name.as_str(),
            layer.blend,
            layer.opacity,
            layer.visible,
            layer.clip_to_below,
            layer.mask.as_ref(),
        ),
        Record::GroupStart(group) => (
            empty_rect,
            group.name.as_str(),
            group.blend,
            group.opacity,
            group.visible,
            false,
            group.mask.as_ref(),
        ),
        Record::GroupEnd => (
            empty_rect,
            "</Layer group>",
            Blend::Normal,
            255,
            true,
            false,
            None,
        ),
    };
    let mut extra = Vec::new();
    if let Some(mask) = mask {
        extra.extend_from_slice(&20u32.to_be_bytes());
        rect_bytes(mask.rect, &mut extra);
        extra.push(mask.default);
        extra.push(if mask.disabled { 0x02 } else { 0 });
        extra.extend_from_slice(&[0, 0]);
    } else {
        extra.extend_from_slice(&0u32.to_be_bytes());
    }
    extra.extend_from_slice(&40u32.to_be_bytes());
    for _ in 0..5 {
        extra.extend_from_slice(&[0, 0, 255, 255, 0, 0, 255, 255]);
    }
    let pascal: Vec<u8> = name
        .chars()
        .take(31)
        .map(|c| if c.is_ascii() { c as u8 } else { b'?' })
        .collect();
    extra.push(u8::try_from(pascal.len()).map_err(|_| Error::Malformed("layer name"))?);
    extra.extend_from_slice(&pascal);
    pad(&mut extra, 4);
    let mut name_data = unicode(name)?;
    pad(&mut name_data, 4);
    tagged(&mut extra, b"luni", &name_data)?;
    tagged(&mut extra, b"lyid", &id.to_be_bytes())?;
    match record {
        Record::GroupStart(group) => {
            let mut divider = Vec::new();
            let kind = if group.expanded { 1u32 } else { 2u32 };
            divider.extend_from_slice(&kind.to_be_bytes());
            divider.extend_from_slice(b"8BIM");
            divider.extend_from_slice(&group.blend.key());
            tagged(&mut extra, b"lsct", &divider)?;
        }
        Record::GroupEnd => tagged(&mut extra, b"lsct", &3u32.to_be_bytes())?,
        Record::Layer(_) => {
            tagged(&mut extra, b"clbl", &[1, 0, 0, 0])?;
            tagged(&mut extra, b"infx", &[0; 4])?;
            tagged(&mut extra, b"knko", &[0; 4])?;
        }
    }

    rect_bytes(rect, bytes);
    let channel_count =
        u16::try_from(channel_ids.len()).map_err(|_| Error::Unsupported("channel count"))?;
    bytes.extend_from_slice(&channel_count.to_be_bytes());
    for channel_id in channel_ids {
        bytes.extend_from_slice(&channel_id.to_be_bytes());
        length_fields.push(bytes.len());
        write_length(bytes, 0, format)?;
    }
    bytes.extend_from_slice(b"8BIM");
    bytes.extend_from_slice(&blend.key());
    bytes.push(opacity);
    bytes.push(u8::from(clipped));
    let mut flags = if visible { 0x08 } else { 0x0a };
    if !matches!(record, Record::Layer(_)) {
        flags |= 0x10;
    }
    bytes.extend_from_slice(&[flags, 0]);
    bytes.extend_from_slice(&length(extra.len())?.to_be_bytes());
    bytes.extend_from_slice(&extra);
    Ok(())
}

#[allow(
    clippy::arithmetic_side_effects,
    clippy::cast_possible_truncation,
    reason = "u8 inputs keep every step inside u32 and the quotient at most 255"
)]
fn over_white(colour: u8, alpha: u8) -> u8 {
    let (c, a) = (u32::from(colour), u32::from(alpha));
    ((c * a + 255 * (255 - a) + 127) / 255) as u8
}
