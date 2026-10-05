//! PSD/PSB writer.

use crate::{Blend, Document, Error, Format, Group, Layer, Node, Rect, Result, rle, validate};

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

struct Channel {
    id: i16,
    data: Vec<u8>,
}

fn prepare_channels(records: &[Record<'_>]) -> Result<Vec<Vec<Channel>>> {
    records.iter().map(record_channels).collect()
}

fn record_channels(record: &Record<'_>) -> Result<Vec<Channel>> {
    let mut channels = Vec::new();
    match record {
        Record::Layer(layer) if !layer.pixels.rect.is_empty() => {
            for (id, offset) in [(-1, 3), (0, 0), (1, 1), (2, 2)] {
                let plane: Vec<u8> = layer
                    .pixels
                    .data
                    .chunks_exact(4)
                    .map(|pixel| pixel[offset])
                    .collect();
                channels.push(Channel {
                    id,
                    data: encode_plane(&plane, layer.pixels.rect)?,
                });
            }
        }
        _ => {
            for id in [-1, 0, 1, 2] {
                channels.push(Channel {
                    id,
                    data: vec![0, 0],
                });
            }
        }
    }
    let mask = match record {
        Record::Layer(layer) => layer.mask.as_ref(),
        Record::GroupStart(group) => group.mask.as_ref(),
        Record::GroupEnd => None,
    };
    if let Some(mask) = mask {
        channels.push(Channel {
            id: -2,
            data: encode_plane(&mask.data, mask.rect)?,
        });
    }
    Ok(channels)
}

// Requires a validated plane buffer.
fn encode_plane(data: &[u8], rect: Rect) -> Result<Vec<u8>> {
    let width = rect.width();
    let mut channel = Vec::from(1u16.to_be_bytes());
    let mut rows = Vec::new();
    for y in 0..rect.height() {
        let start = rows.len();
        rle::encode_row(&data[y * width..(y + 1) * width], &mut rows);
        let count =
            u16::try_from(rows.len() - start).map_err(|_| Error::Unsupported("RLE row length"))?;
        channel.extend_from_slice(&count.to_be_bytes());
    }
    channel.extend_from_slice(&rows);
    Ok(channel)
}

pub fn write(doc: &Document, format: Format, out: &mut impl std::io::Write) -> Result<()> {
    validate::validate(doc, format)?;
    let mut records = Vec::new();
    flatten(&doc.layers, &mut records);
    let count = i16::try_from(records.len()).map_err(|_| Error::Unsupported("layer count"))?;
    let channels = prepare_channels(&records)?;
    let mut layer_records = Vec::new();
    let mut layer_info_len = 2usize;
    for (index, (record, channels)) in records.iter().zip(&channels).enumerate() {
        let bytes = encode_record(record, channels, index as u32 + 1)?;
        layer_info_len = add_length(layer_info_len, bytes.len())?;
        layer_records.push(bytes);
        for channel in channels {
            layer_info_len = add_length(layer_info_len, channel.data.len())?;
        }
    }
    let padding = (4 - layer_info_len % 4) % 4;
    let layer_info_len = length(add_length(layer_info_len, padding)?)?;
    let layer_mask_len = layer_info_len
        .checked_add(8)
        .ok_or(Error::Unsupported("section length"))?;
    let resources = image_resources()?;
    let resources_len = length(resources.len())?;
    let merged = merged_channels(doc)?;

    out.write_all(b"8BPS")?;
    out.write_all(&1u16.to_be_bytes())?;
    out.write_all(&[0; 6])?;
    out.write_all(&4u16.to_be_bytes())?;
    out.write_all(&doc.height.to_be_bytes())?;
    out.write_all(&doc.width.to_be_bytes())?;
    out.write_all(&8u16.to_be_bytes())?;
    out.write_all(&3u16.to_be_bytes())?;
    out.write_all(&0u32.to_be_bytes())?;
    out.write_all(&resources_len.to_be_bytes())?;
    out.write_all(&resources)?;
    out.write_all(&layer_mask_len.to_be_bytes())?;
    out.write_all(&layer_info_len.to_be_bytes())?;
    out.write_all(&(-count).to_be_bytes())?;
    for bytes in layer_records {
        out.write_all(&bytes)?;
    }
    for channels in channels {
        for channel in channels {
            out.write_all(&channel.data)?;
        }
    }
    out.write_all(&[0; 3][..padding])?;
    out.write_all(&0u32.to_be_bytes())?;

    out.write_all(&1u16.to_be_bytes())?;
    let row_counts_end = 2 + doc.height as usize * 2;
    for channel in &merged {
        out.write_all(&channel[2..row_counts_end])?;
    }
    for channel in &merged {
        out.write_all(&channel[row_counts_end..])?;
    }
    Ok(())
}

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

fn add_length(left: usize, right: usize) -> Result<usize> {
    left.checked_add(right)
        .ok_or(Error::Unsupported("section length"))
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

fn image_resources() -> Result<Vec<u8>> {
    let mut data = Vec::new();
    data.extend_from_slice(&1u32.to_be_bytes());
    data.push(1);
    data.extend_from_slice(&unicode("softpsd")?);
    data.extend_from_slice(&unicode("softpsd")?);
    data.extend_from_slice(&1u32.to_be_bytes());
    let mut resource = Vec::new();
    resource.extend_from_slice(b"8BIM");
    resource.extend_from_slice(&1057u16.to_be_bytes());
    resource.extend_from_slice(&[0, 0]);
    resource.extend_from_slice(&length(data.len())?.to_be_bytes());
    resource.extend_from_slice(&data);
    pad(&mut resource, 2);
    Ok(resource)
}

fn encode_record(record: &Record<'_>, channels: &[Channel], id: u32) -> Result<Vec<u8>> {
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
        extra.push((u8::from(mask.disabled) * 0x02) | (u8::from(mask.inverted) * 0x04));
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
    extra.push(pascal.len() as u8);
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
        Record::Layer(_) => {}
    }

    let mut bytes = Vec::new();
    rect_bytes(rect, &mut bytes);
    bytes.extend_from_slice(&(channels.len() as u16).to_be_bytes());
    for channel in channels {
        bytes.extend_from_slice(&channel.id.to_be_bytes());
        bytes.extend_from_slice(&length(channel.data.len())?.to_be_bytes());
    }
    bytes.extend_from_slice(b"8BIM");
    bytes.extend_from_slice(&blend.key());
    bytes.push(opacity);
    bytes.push(u8::from(clipped));
    let mut flags = 0x08 | (u8::from(!visible) * 0x02);
    if !matches!(record, Record::Layer(_)) {
        flags |= 0x10;
    }
    bytes.extend_from_slice(&[flags, 0]);
    bytes.extend_from_slice(&length(extra.len())?.to_be_bytes());
    bytes.extend_from_slice(&extra);
    Ok(bytes)
}

fn merged_channels(doc: &Document) -> Result<Vec<Vec<u8>>> {
    (0..4)
        .map(|offset| {
            let plane: Vec<u8> = doc
                .merged
                .data
                .chunks_exact(4)
                .map(|pixel| {
                    if offset == 3 {
                        pixel[3]
                    } else {
                        let c = u32::from(pixel[offset]);
                        let a = u32::from(pixel[3]);
                        ((c * a + 255 * (255 - a) + 127) / 255) as u8
                    }
                })
                .collect();
            encode_plane(&plane, doc.merged.rect)
        })
        .collect()
}
