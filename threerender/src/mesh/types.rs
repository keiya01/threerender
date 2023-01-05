#[derive(Default, Hash, PartialEq, Eq, Debug)]
pub enum MeshType {
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
