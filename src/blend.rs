//! Blend mode keys.

use crate::Blend;

impl Blend {
    pub fn key(self) -> [u8; 4] {
        todo!("blend key table")
    }

    pub fn from_key(_key: &[u8; 4]) -> Option<Blend> {
        todo!("blend key lookup")
    }
}
