//! Member names and field paths.

mod key;
mod path;

pub use self::key::{Key, ParseKeyError};
pub use self::path::{Path, Segment};

use super::collections::Set;

pub struct Fields<'v>(pub (crate) Option<&'v Set>);


impl<'v> Fields<'v> {
    pub fn field(&self, name: &str) -> bool {
        self.0.map_or(true, |f| f.contains(name))
    }
}