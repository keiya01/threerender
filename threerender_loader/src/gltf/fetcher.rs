use std::rc::Rc;

use threerender_traits::{image::Image, types::Buffer};

use crate::fetcher::err::FetcherError;

/// A trait to handle I/O process.
pub trait GltfFetcher {
    fn fetch(&self, uri: &str) -> Result<Buffer, FetcherError>;
    fn parse_data_url(&self, uri: &str) -> Result<Buffer, FetcherError>;
    fn load_image(&mut self, _buf: Buffer) -> Result<Rc<dyn Image>, FetcherError> {
        Err(FetcherError::TextureNotSupported)
    }
}
