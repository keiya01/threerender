#[derive(thiserror::Error, Debug)]
pub enum GltfError {
    #[error("glTF loader error: {0}")]
    Loader(#[from] gltf::Error),
    #[error("glTF fetcher error: {0}")]
    Fetcher(#[from] std::io::Error),
    #[error("Blob could not find")]
    MissingBlob,
}
