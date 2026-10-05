//! Read and write Adobe Photoshop PSD and PSB files.
//!
//! Write path first. The reader covers what the writer emits and refuses the rest.

#![forbid(unsafe_code)]

pub mod blend;
pub mod model;
pub mod read;
pub mod rle;
pub mod validate;
pub mod write;

pub use model::{Blend, Channels, Document, Group, Image, Layer, Mask, Node, Rect};
pub use write::{format_for, write};

/// 8-bit RGB or RGBA layer data. Other depths and modes are refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Up to 30,000 px per side.
    Psd,
    /// Up to 300,000 px per side.
    Psb,
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Unsupported(&'static str),
    Malformed(&'static str),
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
            Error::Malformed(s) => write!(f, "malformed: {s}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
