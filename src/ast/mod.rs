mod declarations;
mod expressions;
mod statements;

use std::fmt;

use crate::fmt::FormatterExt;
pub use declarations::*;
pub use expressions::*;
pub use statements::*;

/// The contents of a sprachli file. The top-level constructs that can be found
/// in a sprachli file are [Declaration]s.
#[derive(Clone, PartialEq, Eq)]
pub struct SourceFile<'input> {
    pub declarations: Vec<Declaration<'input>>,
}

impl fmt::Debug for SourceFile<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_prefixed().name("sprachli").items(&self.declarations).finish()
    }
}
