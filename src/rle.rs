//! PackBits row encoding and decoding.

use crate::{Error, Result};

pub fn encode_row(row: &[u8], out: &mut Vec<u8>) {
    let mut pos = 0;
    while pos < row.len() {
        let mut run = 1;
        while run < 128 && pos + run < row.len() && row[pos + run] == row[pos] {
            run += 1;
        }
        if run >= 3 {
            out.push((1 - run as i16) as i8 as u8);
            out.push(row[pos]);
            pos += run;
        } else {
            let start = pos;
            while pos < row.len() && pos - start < 128 {
                if pos + 2 < row.len() && row[pos] == row[pos + 1] && row[pos] == row[pos + 2] {
                    break;
                }
                pos += 1;
            }
            out.push((pos - start - 1) as u8);
            out.extend_from_slice(&row[start..pos]);
        }
    }
}

pub fn decode_row(src: &[u8], width: usize, out: &mut Vec<u8>) -> Result<()> {
    let end = out
        .len()
        .checked_add(width)
        .ok_or(Error::Malformed("RLE row"))?;
    let mut pos = 0;
    while out.len() < end {
        let header = *src.get(pos).ok_or(Error::Malformed("RLE row"))? as i8;
        pos += 1;
        match header {
            0..=127 => {
                let count = header as usize + 1;
                if count > end - out.len() || count > src.len() - pos {
                    return Err(Error::Malformed("RLE row"));
                }
                out.extend_from_slice(&src[pos..pos + count]);
                pos += count;
            }
            -127..=-1 => {
                let count = (1 - i16::from(header)) as usize;
                if count > end - out.len() {
                    return Err(Error::Malformed("RLE row"));
                }
                let value = *src.get(pos).ok_or(Error::Malformed("RLE row"))?;
                pos += 1;
                out.resize(out.len() + count, value);
            }
            -128 => {}
        }
    }
    if src[pos..].iter().any(|&byte| byte != 0x80) {
        return Err(Error::Malformed("RLE row"));
    }
    Ok(())
}
