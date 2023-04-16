#[derive(Default, Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Topology {
    PointList,
    LineList,
    #[default]
    TriangleList,
}

#[derive(Default, Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub enum PolygonMode {
    #[default]
    Fill,
    Line,
    Point,
}

#[derive(Hash, Default, PartialEq, Eq, Debug, Clone, Copy)]
pub enum MeshType {
    #[default]
    Entity,
    Texture,
}

pub enum TextureFormat {
    RGBA,
}

pub struct TextureDescriptor {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub data: Vec<u8>,
}
