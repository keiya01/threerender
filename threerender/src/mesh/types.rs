#[derive(Default, Hash, PartialEq, Debug)]
pub enum MeshType {
    PointList,
    LineList,
    #[default]
    TriangleList,
}
