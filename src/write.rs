//! PSD/PSB writer.

use crate::{Document, Error, Format, Group, Layer, Node, Result};

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

pub fn write(_doc: &Document, _format: Format, _out: &mut impl std::io::Write) -> Result<()> {
    Err(Error::Unsupported("not implemented"))
}

pub fn format_for(_width: u32, _height: u32) -> Format {
    todo!("format selection")
}
