use super::{Doc, Parsed};
use crate::doc::indexed::SearchKey;
use rustdoc_types::{Crate, Trait};

impl Doc<Parsed> {
    pub(super) fn search_keys_traits(
        krate: &Crate,
        trait_item: &Trait,
        base_path: &str,
    ) -> Vec<SearchKey> {
        trait_item
            .items
            .iter()
            .filter_map(move |item_id| {
                let item = krate.index.get(item_id)?;
                let name = item.name.as_deref()?;
                Some(SearchKey {
                    id: item_id.0.to_string(),
                    key: format!("{base_path}::{name}"),
                })
            })
            .collect()
    }
}
