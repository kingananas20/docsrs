#![warn(missing_docs)]

//! # docsrs
//!
//! **docsrs** is a Rust crate for retrieving, parsing, and indexing `rustdoc` JSON files,
//! enabling fuzzy searching of Rust documentation content with a type-safe pipeline approach.
//!
//! Whether you're building a custom documentation browser, writing a Rust LSP plugin,
//! creating documentation analysis tools, or just need to programmatically explore docs —
//! `docsrs` gives you structured access to items and metadata in a searchable form.
//!
//! ## Features
//!
//! The crate uses a type-state pattern to ensure compile-time safety when processing
//! documentation through different stages:
//!
//! - **`default`** - Core functionality for loading and parsing JSON files
//! - **`decompress`** - Adds support for decompressing zstd-compressed files
//! - **`fetch`** - Enables fetching compressed documentation directly from docs.rs
//!
//! ## Type-State Pipeline
//!
//! The crate uses a type-state pattern to ensure you process documentation in the correct order:
//!
//! ```text
//! Flow:
//!   Remote ── fetch() ─→ Compressed ── decompress() ─→ RawJson ── parse() ─→ Parsed ── build_search_index() ─→ Indexed
//!                ↑                ↑                         ↑
//!         from_docs()      from_zst()                from_json()
//! ```
//!
//! Each state represents a different stage in the documentation processing pipeline:
//!
//! - **[`Remote`]** - Documentation URL ready to be fetched from docs.rs
//! - **[`Compressed`]** - Downloaded or loaded compressed documentation data
//! - **[`RawJson`]** - Decompressed JSON data in bytes
//! - **[`Parsed`]** - Parsed documentation AST with structured data
//! - **[`Indexed`]** - Documentation with built search index for fuzzy matching
//!
//! ## Search Capabilities
//!
//! The fuzzy search functionality supports:
//!
//! - **Fully qualified paths**: `"std::collections::HashMap"`
//! - **Partial matches**: `"vec push"` matches `Vec::push`
//! - **Case-insensitive**: `"hashmap"` matches `HashMap`
//! - **Methods and functions**: `"tokio::spawn"` finds the spawn function
//! - **Ranked results**: Results are sorted by relevance score
//!
//! ## Item Information
//!
//! Each search result provides comprehensive information:
//!
//! ```rust,ignore
//! # fn main() -> Result<(), docsrs::Error> {
//! # use docsrs::Doc;
//! # let doc = Doc::from_json("example.json")?.parse()?.build_search_index();
//! let results = doc.search("HashMap::new", Some(1));
//! if let Some(items) = results {
//!     for item in items {
//!         println!("Name: {}", item.name);
//!         println!("Path: {}", item.path.join("::"));
//!         println!("Docs: {}", item.docs.as_deref().unwrap_or("No docs"));
//!         println!("Deprecated: {}", item.deprecation.is_some());
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! All operations return `Result<T, Error>` where [`Error`] covers:
//!
//! - HTTP errors when fetching from docs.rs
//! - File I/O errors when reading local files
//! - JSON parsing errors for malformed documentation
//! - URL parsing errors for invalid docs.rs URLs
//!
//! ## Performance Considerations
//!
//! - **Memory usage**: Large crates like `std` can use significant memory when indexed
//! - **Search speed**: Fuzzy search is optimized but may be slower on very large indices
//! - **Parsing time**: Initial parsing and indexing can take time for large documentation sets
//!
//! Consider using search result limits and caching indexed documentation for better performance.
//!
//! ## Examples
//!
//! ### Building a Documentation Browser
//!
//! ```rust,ignore
//! use docsrs::Doc;
//!
//! fn search_docs(query: &str) -> Result<(), Box<dyn std::error::Error>> {
//!     let doc = Doc::from_docs("tokio", "latest")?
//!         .fetch()?
//!         .decompress()?
//!         .parse()?
//!         .build_search_index();
//!
//!     if let Some(results) = doc.search(query, Some(20)) {
//!         for item in results {
//!             println!("{}", item.path.join("::"));
//!             if let Some(docs) = &item.docs {
//!                 println!("  {}", docs.lines().next().unwrap_or(""));
//!             }
//!         }
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ### Analyzing Documentation Coverage
//!
//! ```rust,ignore
//! use docsrs::Doc;
//!
//! fn analyze_coverage(crate_name: &str) -> Result<(), Box<dyn std::error::Error>> {
//!     let doc = Doc::from_docs(crate_name, "latest")?
//!         .fetch()?
//!         .decompress()?
//!         .parse()?
//!         .build_search_index();
//!
//!     let all_items = doc.search("", None).unwrap_or_default();
//!     let documented = all_items.iter().filter(|item| item.docs.is_some()).count();
//!
//!     println!("Total items: {}", all_items.len());
//!     println!("Documented: {}", documented);
//!     println!("Coverage: {:.1}%", (documented as f64 / all_items.len() as f64) * 100.0);
//!
//!     Ok(())
//! }
//! ```
//!
//! This was generated by Claude Sonnet 4, because:
//!
//! - I'm really bad at writing documentation (everything would be very short one-liners)
//! - I'm lazy
//! - I don't have the nerves to write a documentation this long
//!
//! The code was written by me

mod doc;
mod error;

pub use doc::Doc;

pub use doc::Item;
pub use doc::{Indexed, Parsed, RawJson};

#[cfg(feature = "fetch")]
pub use doc::Remote;

pub use doc::Compressed;

pub use error::Error;

// logging for tests
#[cfg(test)]
pub(crate) mod logging {
    use std::sync::Once;

    static INIT_LOGGER: Once = Once::new();

    pub fn init_logger() {
        INIT_LOGGER.call_once(|| {
            env_logger::builder()
                .format_timestamp(None)
                .filter_level(log::LevelFilter::Info)
                .is_test(true)
                .init();
        });
    }
}
