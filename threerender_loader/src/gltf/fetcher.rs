pub type Buffer = Vec<u8>;

pub trait GltfFetcher {
    fn fetch(&self, uri: &str) -> std::io::Result<Buffer>;
    fn parse_data_url(&self, uri: &str) -> std::io::Result<Buffer>;
}
