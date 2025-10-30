use super::{Doc, Parsed};
use crate::doc::indexed::SearchKey;
use rustdoc_types::{Crate, Id, ItemEnum, Struct};
use std::collections::HashMap;

impl Doc<Parsed> {
    pub(super) fn search_keys_structs<'a>(
        &'a self,
        krate: &'a Crate,
        strukt: &'a Struct,
        base_path: &str,
        parent_map: &HashMap<&'a Id, &'a Id>,
        path_cache: &mut HashMap<&'a Id, Vec<String>>,
    ) -> Vec<SearchKey> {
        strukt
            .impls
            .iter()
            .filter_map(move |impl_id| {
                let impl_item = krate.index.get(impl_id)?;

                let ItemEnum::Impl(impl_block) = &impl_item.inner else {
                    return None;
                };

                Some(
                    self.impl_method_keys(krate, impl_block, base_path, parent_map, path_cache)
                        .into_iter(),
                )
            })
            .flatten()
            .collect()
    }
}
