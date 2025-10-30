mod search;

use super::Doc;
use crate::{Error, Item};
use std::{collections::HashMap, fs::OpenOptions, io::Write, path::Path};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SearchKey {
    pub(crate) id: String,
    pub(crate) key: String,
}

/// Represents indexed documentation data with fuzzy search capabilities.
///
/// This struct holds the final processed documentation with a built search index
/// that enables fast fuzzy matching across all documentation items. The index
/// contains searchable keys for all items including their fully qualified paths,
/// methods, and associated functions.
pub struct Indexed {
    pub(crate) search_index: Vec<SearchKey>,
    items: HashMap<String, Item>,
    matcher: fuzzy_matcher::skim::SkimMatcherV2,
}

impl Doc<Indexed> {
    pub(super) fn new(search_index: Vec<SearchKey>, items: HashMap<String, Item>) -> Self {
        Self(Indexed {
            search_index,
            items,
            matcher: fuzzy_matcher::skim::SkimMatcherV2::default(),
        })
    }

    /// Saves the search index to a file for debugging or inspection.
    ///
    /// Writes all search keys to a text file, with each key on a separate line
    /// in debug format. This is useful for examining the generated search index
    /// or debugging search functionality.
    ///
    /// # Arguments
    ///
    /// - `path` - Path where the index file should be written
    ///
    /// # Returns
    ///
    /// `Result<(), Error>` - Success or file I/O error.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # fn main() -> Result<(), docsrs::Error> {
    /// use docsrs::Doc;
    /// let parsed_doc = Doc::from_json("path/to/docs.json")?.parse()?;
    /// let indexed_doc = parsed_doc.build_search_index();
    /// indexed_doc.save_index("debug_index.txt")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn save_index<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)?;

        self.0
            .search_index
            .iter()
            .try_for_each(|key| writeln!(file, "{key:?}"))?;

        Ok(())
    }
}
