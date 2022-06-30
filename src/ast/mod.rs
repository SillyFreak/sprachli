mod declarations;
mod expressions;
mod statements;

pub use declarations::*;
pub use expressions::*;
pub use statements::*;

/// The contents of a sprachli file. The top-level constructs that can be found
/// in a sprachli file are [Declaration]s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub declarations: Vec<Declaration>,
}
