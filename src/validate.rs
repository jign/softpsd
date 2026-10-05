//! Document validation before output.

use crate::{Blend, Document, Error, Format, Mask, Node, Rect, Result};

pub fn validate(doc: &Document, format: Format) -> Result<()> {
    let side_limit = match format {
        Format::Psd => 30_000,
        Format::Psb => 300_000,
    };
    if !(1..=side_limit).contains(&doc.width) || !(1..=side_limit).contains(&doc.height) {
        return Err(Error::Unsupported("side limit"));
    }

    let merged_rect = Rect {
        top: 0,
        left: 0,
        bottom: doc.height as i32,
        right: doc.width as i32,
    };
    if doc.merged.rect != merged_rect
        || area(merged_rect)?.checked_mul(4) != Some(doc.merged.data.len())
    {
        return Err(Error::Malformed("merged image"));
    }

    validate_nodes(&doc.layers, 0)?;
    validate_rect(doc.merged.rect)?;
    validate_rects(&doc.layers)?;

    if let Some(dpi) = doc.resolution_dpi {
        let fixed = (f64::from(dpi) * 65_536.0).round();
        if !dpi.is_finite() || dpi <= 0.0 || fixed < 1.0 || fixed > f64::from(u32::MAX) {
            return Err(Error::Malformed("resolution"));
        }
    }
    Ok(())
}

fn area(rect: Rect) -> Result<usize> {
    rect.area().ok_or(Error::Malformed("rect"))
}

fn validate_nodes(nodes: &[Node], depth: usize) -> Result<()> {
    if depth > 64 {
        return Err(Error::Malformed("nesting"));
    }
    for node in nodes {
        match node {
            Node::Layer(layer) => {
                if area(layer.pixels.rect)?.checked_mul(4) != Some(layer.pixels.data.len()) {
                    return Err(Error::Malformed("layer pixels"));
                }
                if layer.blend == Blend::PassThrough {
                    return Err(Error::Malformed("pass through on a layer"));
                }
                if let Some(mask) = &layer.mask {
                    validate_mask(mask)?;
                }
            }
            Node::Group(group) => {
                if let Some(mask) = &group.mask {
                    validate_mask(mask)?;
                }
                validate_nodes(&group.children, depth + 1)?;
            }
        }
    }
    Ok(())
}

fn validate_mask(mask: &Mask) -> Result<()> {
    if area(mask.rect)? != mask.data.len() || !matches!(mask.default, 0 | 255) {
        return Err(Error::Malformed("mask"));
    }
    Ok(())
}

fn validate_rect(rect: Rect) -> Result<()> {
    if rect.bottom < rect.top || rect.right < rect.left {
        return Err(Error::Malformed("rect"));
    }
    Ok(())
}

fn validate_rects(nodes: &[Node]) -> Result<()> {
    for node in nodes {
        match node {
            Node::Layer(layer) => {
                validate_rect(layer.pixels.rect)?;
                if let Some(mask) = &layer.mask {
                    validate_rect(mask.rect)?;
                }
            }
            Node::Group(group) => {
                if let Some(mask) = &group.mask {
                    validate_rect(mask.rect)?;
                }
                validate_rects(&group.children)?;
            }
        }
    }
    Ok(())
}
