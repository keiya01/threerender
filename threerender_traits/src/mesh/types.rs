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

#[derive(Debug, Clone)]
pub enum TextureFormat {
    Rgba8,
    Rgba16,
}
