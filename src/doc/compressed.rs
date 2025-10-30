use super::{rawjson::RawJson, Doc};
use crate::Error;
use log::debug;
use std::{fs, path::Path};

/// Represents compressed documentation data in zstd format.
///
/// This struct holds compressed bytes that can be decompressed to reveal
/// the raw JSON documentation data.
pub struct Compressed(Vec<u8>);

impl Doc<Compressed> {
    pub(super) fn new(data: Vec<u8>) -> Self {
        Self(Compressed(data))
    }

    /// Loads compressed documentation data from a zstd file.
    ///
    /// Reads a compressed documentation file from disk, typically with a `.zst` extension.
    ///
    /// # Arguments
    ///
    /// - `path` - Path to the compressed documentation file
    ///
    /// # Returns
    ///
    /// `Result<Doc<Compressed>, Error>` - Compressed documentation or file I/O error.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # fn main() -> Result<(), docsrs::Error> {
    /// use docsrs::Doc;
    /// let compressed_doc = Doc::from_zst("docs/serde.json.zst")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_zst<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let compressed_data = fs::read(path)?;
        Ok(Self(Compressed(compressed_data)))
    }

    fn is_compressed(data: &[u8]) -> bool {
        use zstd::zstd_safe::zstd_sys::ZSTD_MAGICNUMBER;
        debug!("{:?}", &data[..4]);

        data.len() >= 4
            && u32::from_le_bytes(data[..4].try_into().unwrap_or([0; 4])) == ZSTD_MAGICNUMBER
    }

    /// Decompresses the documentation data to raw JSON.
    ///
    /// Decompresses the zstd-compressed bytes to reveal the raw JSON documentation data.
    /// Handles multiple layers of compression if present.
    ///
    /// # Returns
    ///
    /// `Result<Doc<RawJson>, Error>` - Raw JSON documentation or decompression error.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # fn main() -> Result<(), docsrs::Error> {
    /// use docsrs::Doc;
    /// let compressed_doc = Doc::from_zst("docs/serde.json.zst")?;
    /// let raw_json = compressed_doc.decompress()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn decompress(self) -> Result<Doc<RawJson>, Error> {
        use std::io::Read;

        let mut data = self.0 .0;

        while Self::is_compressed(&data[..4]) {
            let mut decoder = zstd::Decoder::new(&data[..])?;
            let mut buffer = Vec::new();
            decoder.read_to_end(&mut buffer)?;
            data = buffer;
        }

        Ok(Doc::<RawJson>::new(data))
    }
}
