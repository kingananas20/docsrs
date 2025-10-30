use super::{Doc, compressed::Compressed};
use crate::Error;

/// Represents a remote documentation source that can be fetched from docs.rs.
///
/// This struct holds the URL to a crate's documentation JSON file on docs.rs
/// and provides methods to construct URLs and fetch the documentation data.
pub struct Remote {
    url: url::Url,
}

impl Doc<Remote> {
    /// Creates a new remote documentation reference for the specified crate and version.
    ///
    /// Constructs a URL pointing to the compressed JSON documentation file on docs.rs
    /// for the given crate name and version.
    ///
    /// # Arguments
    ///
    /// - `crate_name` - The name of the crate (e.g., "serde", "tokio")
    /// - `version` - The version string (e.g., "1.0.0", "latest")
    ///
    /// # Returns
    ///
    /// `Result<Doc<Remote>, Error>` - A remote documentation reference or URL parsing error.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # fn main() -> Result<(), docsrs::Error> {
    /// use docsrs::Doc;
    /// let remote_doc = Doc::from_docs("serde", "1.0.193")?;
    /// let remote_doc = Doc::from_docs("tokio", "latest")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_docs(crate_name: &str, version: &str) -> Result<Self, Error> {
        Ok(Doc(Remote {
            url: url::Url::parse(&format!(
                "https://docs.rs/crate/{crate_name}/{version}/json.zst"
            ))?,
        }))
    }

    /// Fetches the documentation data from the remote URL.
    ///
    /// Downloads the compressed JSON documentation file from docs.rs and returns
    /// it as compressed bytes ready for decompression. It uses the ureq crate and
    /// requires the feature `fetch`
    ///
    /// # Returns
    ///
    /// `Result<Doc<Compressed>, Error>` - Compressed documentation data or HTTP/network error.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # fn main() -> Result<(), docsrs::Error> {
    /// use docsrs::Doc;
    /// let remote_doc = Doc::from_docs("serde", "latest")?;
    /// let compressed_doc = remote_doc.fetch()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn fetch(self) -> Result<Doc<Compressed>, Error> {
        let mut res = ureq::get(self.0.url.as_str()).call()?;

        let bytes = res.body_mut().with_config().limit(u64::MAX).read_to_vec()?;

        Ok(<Doc<Compressed>>::new(bytes))
    }
}
