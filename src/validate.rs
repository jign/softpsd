//! Document validation before output.

use crate::{Document, Error, Format, Result};

pub fn validate(_doc: &Document, _format: Format) -> Result<()> {
    // TODO: Map rect area overflow to Error::Malformed("rect").
    Err(Error::Unsupported("not implemented"))
}
