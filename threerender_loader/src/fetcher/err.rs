#[derive(thiserror::Error, Debug)]
pub enum FetcherError {
    #[error("glTF image load error: {0}")]
    Image(#[from] image::ImageError),
    #[error("glTF fetcher error: {0}")]
    Fetcher(#[from] std::io::Error),
    #[error("Texture is not supported")]
    TextureNotSupported,
}
