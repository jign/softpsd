//! PSD/PSB writer.

use crate::{Document, Error, Format, Result};

pub fn write(_doc: &Document, _format: Format, _out: &mut impl std::io::Write) -> Result<()> {
    Err(Error::Unsupported("not implemented"))
}

pub fn format_for(_width: u32, _height: u32) -> Format {
    todo!("format selection")
}
