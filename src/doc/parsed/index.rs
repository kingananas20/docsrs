use super::{Doc, Parsed};
use crate::{Indexed, doc::indexed::SearchKey};
use rustdoc_types::{Crate, Id, Impl, ItemEnum, ItemKind, ItemSummary};
use std::collections::HashMap;

impl Doc<Parsed> {
    /// Builds a fuzzy searchable index from the parsed documentation
    ///
    /// This method processes the parsed documentation AST and creates search keys
    /// for all items (structs, enums, functions, methods, etc.) including their
    /// fully qualified paths. The resulting index enables fast fuzzy searching
    /// across all documentation items.
    ///
    /// # Returns
    ///
    /// A [`Doc<Indexed>`] that supports fuzzy search operations.
    pub fn build_search_index(&self) -> Doc<Indexed> {
        let krate = &self.0.ast;

        // Build a map from child ID to parent module ID to discover re-export paths
        let mut parent_map: HashMap<&Id, &Id> = HashMap::new();
        for (id, item) in &krate.index {
            if let ItemEnum::Module(m) = &item.inner {
                for child_id in &m.items {
                    parent_map.insert(child_id, id);
                }
            }
        }

        // A cache for recursively found paths
        let mut path_cache: HashMap<&Id, Vec<String>> = HashMap::new();

        let mut index: Vec<SearchKey> = krate
            .paths
            .iter()
            .filter_map(|(id, item)| {
                self.generate_searchkeys(id, item, &parent_map, &mut path_cache)
            })
            .flat_map(|vec| vec.into_iter())
            .collect();

        // Add search keys for re-exports (Use items) that are not in `paths`
        for (id, item) in &krate.index {
            if krate.paths.contains_key(id) {
                continue;
            }
            if let ItemEnum::Use(_) = &item.inner
                && let Some(path) = self.get_item_path_recursive(id, &parent_map, &mut path_cache)
            {
                let key = path.join("::");
                index.push(SearchKey {
                    id: id.0.to_string(),
                    key,
                });
            }
        }

        let items = self.build_items(krate.crate_version.clone(), &index);

        <Doc<Indexed>>::new(index, items)
    }

    fn generate_searchkeys<'a>(
        &'a self,
        id: &'a Id,
        item_summary: &'a ItemSummary,
        parent_map: &HashMap<&'a Id, &'a Id>,
        path_cache: &mut HashMap<&'a Id, Vec<String>>,
    ) -> Option<Vec<SearchKey>> {
        if item_summary.crate_id != 0 {
            return None;
        };

        let krate = &self.0.ast;

        let base_path = item_summary.path.join("::");
        let kind = item_summary.kind;

        let mut search_keys = vec![SearchKey {
            id: id.0.to_string(),
            key: base_path.clone(),
        }];

        match kind {
            ItemKind::Struct => {
                if let Some(item) = krate.index.get(id)
                    && let ItemEnum::Struct(s) = &item.inner
                {
                    search_keys.extend(
                        self.search_keys_structs(krate, s, &base_path, parent_map, path_cache),
                    );
                }
            }
            ItemKind::Enum => {
                if let Some(item) = krate.index.get(id)
                    && let ItemEnum::Enum(e) = &item.inner
                {
                    search_keys.extend(
                        self.search_keys_enums(krate, e, &base_path, parent_map, path_cache),
                    );
                }
            }
            ItemKind::Trait => {
                if let Some(item) = krate.index.get(id)
                    && let ItemEnum::Trait(t) = &item.inner
                {
                    search_keys.extend(Self::search_keys_traits(krate, t, &base_path));
                }
            }
            ItemKind::Union => {
                if let Some(item) = krate.index.get(id)
                    && let ItemEnum::Union(u) = &item.inner
                {
                    search_keys.extend(
                        self.search_keys_unions(krate, u, &base_path, parent_map, path_cache),
                    );
                }
            }
            _ => {}
        }

        Some(search_keys)
    }

    pub(super) fn impl_method_keys<'a>(
        &self,
        krate: &Crate,
        impl_block: &'a Impl,
        base_path: &str,
        parent_map: &HashMap<&'a Id, &'a Id>,
        path_cache: &mut HashMap<&'a Id, Vec<String>>,
    ) -> Vec<SearchKey> {
        let path_to_use = if let Some(trait_path) = &impl_block.trait_ {
            self.get_item_path_recursive(&trait_path.id, parent_map, path_cache)
                .map(|p| p.join("::"))
        } else {
            Some(base_path.to_string())
        };

        let Some(path_to_use) = path_to_use else {
            return Vec::new();
        };

        impl_block
            .items
            .iter()
            .filter_map(move |method_id| {
                let method_item = krate.index.get(method_id)?;
                let name = method_item.name.as_deref()?;
                Some(SearchKey {
                    id: method_id.0.to_string(),
                    key: format!("{path_to_use}::{name}"),
                })
            })
            .collect()
    }

    /// Recursively finds the path of an item by traversing up the module tree.
    /// This is a fallback for items not present in the `paths` map, like re-exports.
    pub(super) fn get_item_path_recursive<'a>(
        &self,
        id: &'a Id,
        parent_map: &HashMap<&'a Id, &'a Id>,
        cache: &mut HashMap<&'a Id, Vec<String>>,
    ) -> Option<Vec<String>> {
        if let Some(path) = cache.get(id) {
            return Some(path.clone());
        }
        if let Some(summary) = self.0.ast.paths.get(id) {
            cache.insert(id, summary.path.clone());
            return Some(summary.path.clone());
        }

        let parent_id = parent_map.get(id)?;
        let mut path = self.get_item_path_recursive(parent_id, parent_map, cache)?;

        let item = self.0.ast.index.get(id)?;
        let name = match &item.inner {
            ItemEnum::Use(u) => &u.name,
            _ => item.name.as_deref()?,
        };
        path.push(name.to_string());
        cache.insert(id, path.clone());
        Some(path)
    }
}
