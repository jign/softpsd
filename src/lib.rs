//! Read and write Adobe Photoshop PSD and PSB files.
//!
//! Write path first. The reader covers what the writer emits and refuses the rest.

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::todo,
        clippy::unimplemented,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss
    )
)]

mod blend;
mod model;
mod read;
mod rle;
mod validate;
mod write;

pub use model::{Blend, Channels, Document, Group, Image, Layer, Mask, Node, Rect};
pub use read::{Header, read, read_header, read_with_limit};
pub use write::{format_for, write};

/// File format: PSD, or PSB above 30,000 px on either side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Up to 30,000 px per side.
    Psd,
    /// Up to 300,000 px per side.
    Psb,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Io(std::io::Error),
    Unsupported(&'static str),
    UnsupportedLayer {
        name: String,
        reason: &'static str,
    },
    Malformed(&'static str),
    /// Pixel bytes the document would decode to, and the cap passed to `read_with_limit`.
    OverLimit {
        needed: u64,
        limit: u64,
    },
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "io: {e}"),
            Error::Unsupported(s) => write!(f, "unsupported: {s}"),
            Error::UnsupportedLayer { name, reason } => {
                write!(f, "unsupported layer '{name}': {reason}")
            }
            Error::Malformed(s) => write!(f, "malformed: {s}"),
            Error::OverLimit { needed, limit } => {
                write!(f, "decoded size {needed} bytes over the {limit} byte limit")
            }
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
