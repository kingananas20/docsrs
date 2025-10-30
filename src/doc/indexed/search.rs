use super::{Doc, Indexed, SearchKey};
use crate::Item;
use fuzzy_matcher::FuzzyMatcher;

impl Doc<Indexed> {
    /// Performs fuzzy search on the indexed documentation
    ///
    /// Searches through all documentation items using fuzzy matching, returning
    /// results ranked by similarity score. The search is case-insensitive and matches
    /// against fully qualified item paths (e.g., "std::vec::Vec::push").
    ///
    /// # Arguments
    ///
    /// - `query` - The search term to match against
    /// - `n` - Maximum numbers of results to return (None for all matches)
    ///
    /// # Returns
    ///
    /// `Some(Vec<&Item>)` with matching items ranked by relevance, or `None` if matches found.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # fn main() -> Result<(), docsrs::Error> {
    /// use docsrs::Doc;
    /// let indexed_doc = Doc::from_json("path/to/docs.json")?.parse()?.build_search_index();
    /// // Search for up to 5 items matching "vec push"
    /// let results = indexed_doc.search("vec push", Some(5));
    /// let results = indexed_doc.search("vec push", 5); // this works too because `n` is `impl Into<Option<usize>>`
    ///
    /// // Get all matches
    /// let results = indexed_doc.search("HashMap", None);
    /// # Ok(())
    /// # }
    /// ```
    pub fn search(&self, query: &str, n: impl Into<Option<usize>>) -> Option<Vec<&Item>> {
        let index = &self.0.search_index;
        let matcher = &self.0.matcher;
        let lower_query = query.to_lowercase();

        let mut results = index
            .iter()
            .filter_map(|search_key| {
                matcher
                    .fuzzy_match(&search_key.key.to_lowercase(), &lower_query)
                    .map(|score| (score, search_key))
            })
            .collect::<Vec<(i64, &SearchKey)>>();

        results.sort_unstable_by(|a, b| b.0.cmp(&a.0).then(a.1.key.len().cmp(&b.1.key.len())));

        if let Some(item) = results.iter().find_map(|(_, search_key)| {
            if search_key.key.to_lowercase() == lower_query {
                self.0.items.get(&search_key.id).filter(|i| !i.name.is_empty())
            } else {
                None
            }
        }) {
            return Some(vec![item]);
        }

        let n = n.into();
        if n == Some(0) || results.is_empty() {
            return None;
        }

        let items: Vec<_> = results
            .iter()
            .filter_map(|(_, search_key)| self.0.items.get(&search_key.id))
            .filter(|item| !item.name.is_empty())
            .take(n.unwrap_or(usize::MAX))
            .collect();

        if items.is_empty() {
            None
        } else {
            Some(items)
        }
    }
}
