//! Document validation before output.

use crate::{Document, Error, Format, Result};

pub fn validate(_doc: &Document, _format: Format) -> Result<()> {
    Err(Error::Unsupported("not implemented"))
}
