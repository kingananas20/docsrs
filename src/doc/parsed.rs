mod enums;
mod index;
mod item;
mod structs;
mod traits;
mod unions;

use super::Doc;
pub use item::Item;
use rustdoc_types::Crate;

/// Represents parsed documentation data with a structured AST.
///
/// This struct holds the complete rustdoc AST (Abstract Syntax Tree) after
/// JSON deserialization. The AST contains all documentation items, their
/// relationships, and metadata in a structured format ready for indexing
/// and processing.
pub struct Parsed {
    /// The complete rustdoc AST containing all documentation items
    pub ast: Crate,
}

impl Doc<Parsed> {
    pub(super) fn new(ast: Crate) -> Self {
        Self(Parsed { ast })
    }
}
