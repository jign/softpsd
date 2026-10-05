//! PackBits row encoding and decoding.

use crate::{Error, Result};

pub fn encode_row(_row: &[u8], _out: &mut Vec<u8>) {
    todo!("PackBits encoder")
}

pub fn decode_row(_src: &[u8], _width: usize, _out: &mut Vec<u8>) -> Result<()> {
    Err(Error::Unsupported("not implemented"))
}
