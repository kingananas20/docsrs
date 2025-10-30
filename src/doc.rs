#[cfg(feature = "fetch")]
mod remote;
#[cfg(feature = "fetch")]
pub use remote::Remote;

mod compressed;
pub use compressed::Compressed;

mod rawjson;
pub use rawjson::RawJson;

mod parsed;
pub use parsed::{Item, Parsed};

mod indexed;
pub use indexed::Indexed;

/// A generic wrapper for documentation data in different processing states.
///
/// This struct uses the type-state pattern to ensure compile-time safety when
/// transitioning between different stages of documentation processing. Each state
/// represents a different stage in the documentation pipeline.
///
/// # States
///
/// - [`Remote`] - Documentation URL ready to be fetched
/// - [`Compressed`] - Downloaded or opened compressed documentation data
/// - [`RawJson`] - Decompressed JSON data in bytes
/// - [`Parsed`] - Parsed documentation AST
/// - [`Indexed`] - Documentation with searchable index
///
/// # Example
///
/// ```rust,ignore
/// # fn main() -> Result<(), docsrs::Error> {
/// use docsrs::Doc;
///
/// let doc = Doc::from_docs("serde", "latest")?
///     .fetch()?
///     .decompress()?
///     .parse()?
///     .build_search_index();
///
/// let results = doc.search("Serialize", None);
/// # Ok(())
/// # }
/// ```
pub struct Doc<State>(pub State);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::init_logger;

    fn assert_path_superset(superset_path: &[String], subset_path: &[&str]) {
        let is_superset = subset_path
            .iter()
            .all(|item| superset_path.contains(&item.to_string()));
        let same_start = superset_path.first().map(|s| s.as_str()) == subset_path.first().copied();
        let same_end = superset_path.last().map(|s| s.as_str()) == subset_path.last().copied();

        assert!(
            is_superset && same_start && same_end,
            "path mismatch: left=`{superset_path:?}` is not a valid superset of right=`{subset_path:?}`"
        );
    }

    #[test]
    #[cfg(feature = "fetch")]
    fn fetch_tokio() {
        init_logger();

        let krate = Doc::from_docs("tokio", "latest").unwrap();
        let krate = krate.fetch().unwrap();
        let krate = krate.decompress().unwrap();
        let krate = krate.parse().unwrap();
        let krate = krate.build_search_index();

        let hit = krate.search("tokio::spawn", 1).unwrap();
        let item = &hit[0];
        assert_eq!(
            item.name, "spawn",
            "unexpected item name, full item: {item:#?}"
        );
        assert_path_superset(&item.path, &["tokio", "task", "spawn", "spawn"]);
    }

    #[test]
    #[cfg(feature = "fetch")]
    fn fetch_serde() {
        init_logger();

        let krate = Doc::from_docs("serde", "latest").unwrap();
        let krate = krate.fetch().unwrap();
        let krate = krate.decompress().unwrap();
        let krate = krate.parse().unwrap();
        let krate = krate.build_search_index();

        let hits = krate.search("serde::Serialize", 1).unwrap();
        let item = &hits[0];
        assert_eq!(
            item.name, "Serialize",
            "unexpected item name, full item: {item:#?}"
        );
        assert_path_superset(&item.path, &["serde", "Serialize"]);
    }

    #[test]
    fn from_json() {
        init_logger();

        const STD_JSON_PATH_ENV: &str = "RUSTDOC_JSON_STD_PATH";
        let path = if let Ok(path) = std::env::var(STD_JSON_PATH_ENV) {
            path
        } else {
            println!("Skipping test `from_json`: env var `{STD_JSON_PATH_ENV}` not set.");
            println!("Example: `export {STD_JSON_PATH_ENV}=/home/user/.rustup/toolchains/nightly/share/doc/rust/json/std.json`");
            return;
        };

        let std = Doc::from_json(path).unwrap();
        let std = std.parse().unwrap().build_search_index();

        let hit = std.search("std::fs::File", 1);
        println!("{hit:#?}")
    }

    #[test]
    #[cfg(feature = "fetch")]
    fn fetch_lancedb() {
        init_logger();

        let krate = Doc::from_docs("lancedb", "latest").unwrap();
        let krate = krate.fetch().unwrap();
        let krate = krate.decompress().unwrap();
        let krate = krate.parse().unwrap();
        let krate = krate.build_search_index();

        krate.save_index("lancedb_index.txt").unwrap();

        let hits = krate.search("lancedb::table::Table", 1).unwrap();
        let item = &hits[0];
        assert_eq!(
            item.name, "Table",
            "unexpected item name, full item: {item:#?}"
        );
        assert!(item.docs.is_some(), "docs for Table should exist");

        let hits = krate
            .search("lancedb::table::Table::create_index", 5)
            .unwrap();
        let item = hits
            .iter()
            .find(|item| item.name == "create_index")
            .expect("item `create_index` not found in search results");
        assert_eq!(
            item.name, "create_index",
            "unexpected item name, full item: {item:#?}"
        );
        assert!(
            item.docs.is_some(),
            "docs for Table::create_index should exist"
        );
        assert_path_superset(&item.path, &["lancedb", "table", "Table", "create_index"]);
    }
}
