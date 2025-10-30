use super::{Doc, Parsed};
use crate::{Error, doc::indexed::SearchKey};
use rustdoc_types::{Attribute, ItemEnum};
use std::collections::HashMap;
use url::Url;

impl Doc<Parsed> {
    pub(super) fn build_items(
        &self,
        index: &[SearchKey],
        version: Option<String>,
    ) -> HashMap<u32, Item> {
        index
            .iter()
            .map(|key| {
                let path = key.key.split("::").map(|s| s.to_owned()).collect();

                let item = self
                    .0
                    .ast
                    .index
                    .get(&rustdoc_types::Id(key.id))
                    .unwrap()
                    .clone();

                let links = item.links.into_iter().map(|(k, id)| (k, (id.0))).collect();

                (
                    key.id,
                    Item {
                        id: key.id,
                        crate_id: item.crate_id,
                        crate_version: version.clone(),
                        path,
                        visibility: item.visibility,
                        span: item.span,
                        name: item.name.unwrap_or_default(),
                        docs: item.docs,
                        links,
                        attributes: item.attrs,
                        deprecation: item.deprecation,
                        inner: item.inner,
                    },
                )
            })
            .collect()
    }
}

/// Represents a single documentation item with all its metadata and content.
///
/// This struct contains all the information about a documentation item (function,
/// struct, enum, module, etc.) extracted from the rustdoc AST. It provides a
/// simplified and searchable representation of the original rustdoc data with
/// preprocessed paths and normalized identifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// Unique identifier for this item within the documentation
    pub id: u32,
    /// Identifier of the crate this item belongs to
    pub crate_id: u32,
    /// The crate version if given
    pub crate_version: Option<String>,
    /// Fully qualified path components (e.g., ["std", "collections", "HashMap"])
    pub path: Vec<String>,
    /// Source code location information, if available
    pub visibility: rustdoc_types::Visibility,
    /// Name of the item (e.g., "HashMap", "push", "main")
    pub span: Option<rustdoc_types::Span>,
    /// Name of the item (e.g., "HashMap", "push", "main")
    pub name: String,
    /// Documentation text content in markdown format
    pub docs: Option<String>,
    /// Links to other documentation items referenced in the docs
    pub links: HashMap<String, u32>,
    /// Rust attributes applied to this item (e.g., "#[derive(Debug)]")
    pub attributes: Vec<Attribute>,
    /// Deprecation information if the item is deprecated
    pub deprecation: Option<rustdoc_types::Deprecation>,
    /// The actual item type and data (struct, enum, function, etc.)
    pub inner: rustdoc_types::ItemEnum,
}

impl Item {
    /// Returns the url for the search site of the crate
    pub fn search_url(&self) -> Result<Url, Error> {
        let path = self.path.join("::");
        Ok(Url::parse(&format!("https://docs.rs/{path}"))?)
    }

    pub(crate) fn _url(&self) -> Result<Option<Url>, Error> {
        if self.path.is_empty() {
            return Ok(None);
        }

        let Some(crate_name) = self.path.first() else {
            return Ok(None);
        };

        let version = self.crate_version.as_deref().unwrap_or("latest");

        let mut url = Url::parse(&format!("https://docs.rs/{crate_name}/{version}"))?;

        match &self.inner {
            ItemEnum::Function(_) => {
                if self.path.len() >= 3 {
                    let parent_path = &self.path[..&self.path.len() - 1];
                    let parent_type = parent_path.last().unwrap();

                    let mut path_segments = url.path_segments_mut().unwrap();
                    for segment in parent_path {
                        path_segments.push(segment);
                    }
                    path_segments.push(&format!("struct.{parent_type}.html"));
                    drop(path_segments);

                    url.set_fragment(Some(&format!("method.{}", self.name)));
                } else {
                    let mut path_segments = url.path_segments_mut().unwrap();
                    for segment in &self.path {
                        path_segments.push(segment);
                    }
                    path_segments.push(&format!("fn.{}.html", self.name));
                }
            }
            _ => return Ok(None),
        }

        Ok(Some(url))
    }
}
