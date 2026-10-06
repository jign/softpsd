#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
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

/// Why a read or write failed.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The writer's output failed.
    Io(std::io::Error),
    /// Valid PSD that softpsd does not handle, with the reason.
    Unsupported(&'static str),
    /// A layer kind softpsd does not handle, such as text or a smart object.
    UnsupportedLayer {
        /// The layer's name, so the user can find it.
        name: String,
        /// What the layer is.
        reason: &'static str,
    },
    /// Broken input: a truncated file, or a document that breaks the model's rules.
    Malformed(&'static str),
    /// The document's pixel buffers would pass the cap given to `read_with_limit`.
    OverLimit {
        /// Pixel bytes the document would decode to.
        needed: u64,
        /// The cap.
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

/// `Result` with softpsd's `Error`.
pub type Result<T> = std::result::Result<T, Error>;
