#[derive(Default, Hash, PartialEq, Eq, Debug)]
pub enum Topology {
    PointList,
    LineList,
    #[default]
    TriangleList,
}

#[derive(Default, Hash, PartialEq, Eq, Debug)]
pub enum PolygonMode {
    #[default]
    Fill,
    Line,
    Point,
}

#[derive(Hash, Default, PartialEq, Eq, Debug)]
pub enum MeshType {
    #[default]
    Entity,
    Texture2D,
}

pub enum Texture2DFormat {
    RGBA,
}

pub struct Texture2DDescriptor {
    pub width: u32,
    pub height: u32,
    pub format: Texture2DFormat,
    pub data: Vec<u8>,
}
