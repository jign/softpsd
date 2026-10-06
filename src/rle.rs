//! PackBits row encoding and decoding.

use crate::{Error, Result};

pub(crate) fn encode_row(row: &[u8], out: &mut Vec<u8>) {
    let mut rest = row;
    while let Some((&first, tail)) = rest.split_first() {
        let repeats = tail
            .iter()
            .take(127)
            .take_while(|&&byte| byte == first)
            .count();
        if repeats >= 2 {
            out.push(repeat_header(repeats));
            out.push(first);
            rest = tail.get(repeats..).unwrap_or_default();
        } else {
            let limit = rest.len().min(128);
            let next = rest.get(1..).unwrap_or_default();
            let after = rest.get(2..).unwrap_or_default();
            let len = rest
                .iter()
                .zip(next)
                .zip(after)
                .take(limit)
                .enumerate()
                .skip(1)
                .find(|(_, ((a, b), c))| a == b && b == c)
                .map_or(limit, |(i, _)| i);
            let (literal, tail) = rest.split_at_checked(len).unwrap_or((rest, &[]));
            out.push(literal_header(len));
            out.extend_from_slice(literal);
            rest = tail;
        }
    }
}

#[allow(
    clippy::arithmetic_side_effects,
    clippy::cast_possible_truncation,
    reason = "repeats is 2..=127, header is 129..=254"
)]
fn repeat_header(repeats: usize) -> u8 {
    (256 - repeats) as u8
}

#[allow(
    clippy::arithmetic_side_effects,
    clippy::cast_possible_truncation,
    reason = "len is 1..=128, header is 0..=127"
)]
fn literal_header(len: usize) -> u8 {
    (len - 1) as u8
}

pub(crate) fn decode_row(src: &[u8], width: usize, out: &mut Vec<u8>) -> Result<()> {
    let malformed = || Error::Malformed("RLE row");
    let mut src = src;
    let mut left = width;
    while left != 0 {
        let (&header, rest) = src.split_first().ok_or_else(malformed)?;
        src = rest;
        match header {
            0..=127 => {
                let count = usize::from(header).checked_add(1).ok_or_else(malformed)?;
                left = left.checked_sub(count).ok_or_else(malformed)?;
                let (literal, rest) = src.split_at_checked(count).ok_or_else(malformed)?;
                out.extend_from_slice(literal);
                src = rest;
            }
            129..=255 => {
                let count = 257usize
                    .checked_sub(usize::from(header))
                    .ok_or_else(malformed)?;
                left = left.checked_sub(count).ok_or_else(malformed)?;
                let (&value, rest) = src.split_first().ok_or_else(malformed)?;
                src = rest;
                out.extend(std::iter::repeat_n(value, count));
            }
            128 => {}
        }
    }
    if src.iter().any(|&byte| byte != 0x80) {
        return Err(malformed());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rle_round_trip() {
        let literals: Vec<u8> = (0..=255).cycle().take(300).collect();
        let mut mixed = vec![42; 100];
        mixed.extend_from_slice(&literals);
        mixed.extend([17; 300]);
        for row in [vec![42; 100], literals, vec![17; 300], mixed, Vec::new()] {
            let mut encoded = Vec::new();
            encode_row(&row, &mut encoded);
            let mut decoded = Vec::new();
            decode_row(&encoded, row.len(), &mut decoded).unwrap();
            assert_eq!(decoded, row);
            encoded.extend([0x80, 0x80]);
            decoded.clear();
            decode_row(&encoded, row.len(), &mut decoded).unwrap();
            assert_eq!(decoded, row);
        }
    }
}
