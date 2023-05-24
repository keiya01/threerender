use crate::fetcher::err::FetcherError;

#[derive(thiserror::Error, Debug)]
pub enum GltfError {
    #[error("glTF loader error: {0}")]
    Loader(#[from] gltf::Error),
    #[error("glTF fetcher error: {0}")]
    Fetcher(#[from] FetcherError),
    #[error("Blob could not find")]
    MissingBlob,
}
