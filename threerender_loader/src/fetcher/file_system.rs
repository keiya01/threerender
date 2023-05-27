use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use base64::Engine;

use crate::gltf::fetcher::{Buffer, GltfFetcher, GltfImage};

use super::{err::FetcherError, image::Image, LoaderFetcher};

pub struct DefaultFileSystemBasedFetcher {
    resolve_path: PathBuf,
}

impl DefaultFileSystemBasedFetcher {
    pub fn with_resolve_path(resolve_path: PathBuf) -> Self {
        Self { resolve_path }
    }
}

impl LoaderFetcher for DefaultFileSystemBasedFetcher {}

impl GltfFetcher for DefaultFileSystemBasedFetcher {
    /// Read data from specified path
    fn fetch(&self, uri: &str) -> Result<crate::gltf::fetcher::Buffer, FetcherError> {
        let path = Path::new(&self.resolve_path).join(uri);
        let mut f = File::open(path)?;

        let mut buf = vec![];
        f.read_to_end(&mut buf)?;

        Ok(buf)
    }

    /// Parse and read Data URLs protocol
    fn parse_data_url(&self, uri: &str) -> Result<crate::gltf::fetcher::Buffer, FetcherError> {
        let uri = percent_encoding::percent_decode_str(uri)
            .decode_utf8()
            .unwrap();
        let uri = match uri.strip_prefix("data:") {
            Some(uri) => uri,
            None => {
                return Err(FetcherError::Fetcher(std::io::Error::from(
                    std::io::ErrorKind::InvalidInput,
                )))
            }
        };
        let mut iter = uri.splitn(2, ',');
        let (mime_type, data) = match (iter.next(), iter.next()) {
            (Some(a), Some(b)) => (a, b),
            _ => {
                return Err(FetcherError::Fetcher(std::io::Error::from(
                    std::io::ErrorKind::InvalidInput,
                )))
            }
        };

        let (_mime_type, is_base64) = match mime_type.strip_suffix(";base64") {
            Some(mime_type) => (mime_type, true),
            None => (mime_type, false),
        };

        if is_base64 {
            match base64::engine::general_purpose::STANDARD.decode(data) {
                Ok(v) => Ok(v),
                // FIXME(@keiya01): logging for base64 parsing error
                Err(_) => Err(FetcherError::Fetcher(std::io::Error::from(
                    std::io::ErrorKind::InvalidData,
                ))),
            }
        } else {
            Ok(data.as_bytes().to_owned())
        }
    }

    /// Load the exact image buffer from the data buffer.
    fn load_image(&mut self, buf: Buffer) -> Result<Box<dyn GltfImage>, FetcherError> {
        let img = image::load_from_memory(&buf)?;
        Ok(Box::new(Image::from_image(img)))
    }
}
