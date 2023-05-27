use threerender_traits::mesh::TextureFormat;

use crate::fetcher::err::FetcherError;

pub type Buffer = Vec<u8>;

/// A trait to handle I/O process.
pub trait GltfFetcher {
    fn fetch(&self, uri: &str) -> Result<Buffer, FetcherError>;
    fn parse_data_url(&self, uri: &str) -> Result<Buffer, FetcherError>;
    fn load_image(&mut self, _buf: Buffer) -> Result<Box<dyn GltfImage>, FetcherError> {
        Err(FetcherError::TextureNotSupported)
    }
}

/// A trait to retrieve necessary image information.
pub trait GltfImage {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn format(&self) -> TextureFormat;
    fn data(&mut self) -> Buffer;
}
