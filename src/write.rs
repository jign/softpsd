//! PSD/PSB writer.

use crate::{Document, Error, Format, Group, Layer, Node, Rect, Result, rle};

#[allow(dead_code)]
enum Record<'a> {
    Layer(&'a Layer),
    GroupEnd,
    GroupStart(&'a Group),
}

#[allow(dead_code)]
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

#[allow(dead_code)]
struct Channel {
    id: i16,
    data: Vec<u8>,
}

#[allow(dead_code)]
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
    if let Record::Layer(layer) = record
        && let Some(mask) = &layer.mask
    {
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

pub fn write(_doc: &Document, _format: Format, _out: &mut impl std::io::Write) -> Result<()> {
    Err(Error::Unsupported("not implemented"))
}

pub fn format_for(_width: u32, _height: u32) -> Format {
    todo!("format selection")
}
