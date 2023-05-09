use std::{fs::File, io::Read, path::{Path, PathBuf}};

use base64::Engine;

use crate::gltf::fetcher::GltfFetcher;

use super::LoaderFetcher;

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
    fn fetch(&self, uri: &str) -> std::io::Result<crate::gltf::fetcher::Buffer> {
        let path = Path::new(&self.resolve_path).join(uri);
        println!("Path: {}", path.to_str().unwrap());
        let mut f = File::open(path)?;

        let mut buf = vec![];
        f.read_to_end(&mut buf)?;

        Ok(buf)
    }

    fn parse_data_url(&self, uri: &str) -> std::io::Result<crate::gltf::fetcher::Buffer> {
        let uri = percent_encoding::percent_decode_str(uri)
            .decode_utf8()
            .unwrap();
        let uri = match uri.strip_prefix("data:") {
            Some(uri) => uri,
            None => return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput)),
        };
        let mut iter = uri.splitn(2, ",");
        let (mime_type, data) = match (iter.next(), iter.next()) {
            (Some(a), Some(b)) => (a, b),
            _ => return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput)),
        };

        let (_mime_type, is_base64) = match mime_type.strip_suffix(";base64") {
            Some(mime_type) => (mime_type, true),
            None => (mime_type, false),
        };

        if is_base64 {
            match base64::engine::general_purpose::STANDARD_NO_PAD.decode(data) {
                Ok(v) => Ok(v),
                // FIXME(@keiya01): logging for base64 parsing error
                Err(_) => Err(std::io::Error::from(std::io::ErrorKind::InvalidData)),
            }
        } else {
            Ok(data.as_bytes().to_owned())
        }
    }
}
