use threerender_traits::mesh::TextureFormat;

#[derive(Debug)]
pub struct TextureDescriptor {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub data: Vec<u8>,
}
