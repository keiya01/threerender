use crate::math::Vec3;

use super::{
    traits::{EntityMesh, Mesh, TextureMesh},
    types::Topology,
    util::{texture, texture_vertex, vertex, TextureVertex, Vertex},
    TextureDescriptor, TextureFormat,
};

pub struct TriangleList {
    vertex: Vec<Vertex>,
    index: Option<Vec<u16>>,
}

impl TriangleList {
    pub fn new(a: Vec3, b: Vec3, c: Vec3) -> Self {
        let normal = Vec3::new(0., 0., 1.0);
        Self {
            vertex: vec![
                Vertex::from_vec3(a, normal),
                Vertex::from_vec3(b, normal),
                Vertex::from_vec3(c, normal),
            ],
            index: None,
        }
    }

    pub fn push_vertex(&mut self, a: Vertex, b: Vertex, c: Vertex) {
        self.vertex.extend_from_slice(&[a, b, c]);
    }

    pub fn push_index(&mut self, v: [u16; 3]) {
        match &mut self.index {
            Some(idx) => idx.extend_from_slice(&v),
            None => self.index = Some(v.to_vec()),
        }
    }
}

impl EntityMesh for TriangleList {
    fn vertex(&self) -> &[Vertex] {
        &self.vertex
    }

    fn index(&self) -> Option<&[u16]> {
        match &self.index {
            Some(idx) => Some(idx),
            None => None,
        }
    }

    fn use_entity(self) -> Mesh {
        Mesh::Entity(Box::new(self))
    }
}

pub enum PointTopology {
    Point,
    Line,
}

pub struct PointList {
    vertex: Vec<Vertex>,
    index: Option<Vec<u16>>,
    mesh_type: PointTopology,
}

impl PointList {
    pub fn new(points: Vec<Vec3>, mesh_type: PointTopology) -> Self {
        Self {
            vertex: Self::points_to_vertex(points),
            index: None,
            mesh_type,
        }
    }

    pub fn push_vertex(&mut self, points: Vec<Vec3>) {
        self.vertex
            .extend_from_slice(&Self::points_to_vertex(points));
    }

    pub fn push_index(&mut self, v: [u16; 3]) {
        match &mut self.index {
            Some(idx) => idx.extend_from_slice(&v),
            None => self.index = Some(v.to_vec()),
        }
    }

    fn points_to_vertex(points: Vec<Vec3>) -> Vec<Vertex> {
        let normal = Vec3::new(0., 0., 1.0);
        points
            .into_iter()
            .map(|p| Vertex::from_vec3(p, normal))
            .collect()
    }
}

impl EntityMesh for PointList {
    fn vertex(&self) -> &[Vertex] {
        &self.vertex
    }

    fn index(&self) -> Option<&[u16]> {
        match &self.index {
            Some(idx) => Some(idx),
            None => None,
        }
    }

    fn topology(&self) -> Topology {
        match &self.mesh_type {
            PointTopology::Point => Topology::PointList,
            PointTopology::Line => Topology::LineList,
        }
    }

    fn use_entity(self) -> Mesh {
        Mesh::Entity(Box::new(self))
    }
}

pub struct Plane {
    vertex: Vec<Vertex>,
    index: [u16; 6],

    texture_descriptor: Option<TextureDescriptor>,
    texture: Option<Vec<TextureVertex>>,
}

impl Plane {
    pub fn new(normal: [i8; 3]) -> Self {
        let mat: [[i8; 3]; 4] = match &normal {
            [1, 0, 0] => [[0, -1, 1], [0, -1, -1], [0, 1, -1], [0, 1, 1]],
            [-1, 0, 0] => [[0, -1, -1], [0, -1, 1], [0, 1, 1], [0, 1, -1]],
            [0, 1, 0] => [[-1, 0, -1], [-1, 0, 1], [1, 0, 1], [1, 0, -1]],
            [0, -1, 0] => [[-1, 0, -1], [1, 0, -1], [1, 0, 1], [-1, 0, 1]],
            [0, 0, 1] => [[-1, -1, 0], [1, -1, 0], [1, 1, 0], [-1, 1, 0]],
            [0, 0, -1] => [[1, -1, 0], [-1, -1, 0], [-1, 1, 0], [1, 1, 0]],
            _ => unimplemented!(),
        };

        let normal = normal.map(|v| v as f32);

        Self {
            vertex: mat
                .map(|v| vertex([v[0] as f32, v[1] as f32, v[2] as f32, 1.], normal))
                .to_vec(),
            index: [0, 1, 2, 0, 2, 3],

            texture_descriptor: None,
            texture: None,
        }
    }
}

impl EntityMesh for Plane {
    fn vertex(&self) -> &[Vertex] {
        &self.vertex
    }

    fn index(&self) -> Option<&[u16]> {
        Some(&self.index)
    }

    fn use_entity(self) -> Mesh {
        Mesh::Entity(Box::new(self))
    }
}

impl TextureMesh for Plane {
    fn texture(&self) -> Option<&[TextureVertex]> {
        self.texture.as_ref().map(|t| &t[..])
    }
    fn width(&self) -> u32 {
        self.texture_descriptor.as_ref().unwrap().width
    }
    fn height(&self) -> u32 {
        self.texture_descriptor.as_ref().unwrap().height
    }
    fn format(&self) -> &TextureFormat {
        &self.texture_descriptor.as_ref().unwrap().format
    }
    fn data(&self) -> &[u8] {
        &self.texture_descriptor.as_ref().unwrap().data
    }

    fn use_texture(mut self, descriptor: TextureDescriptor) -> Mesh {
        let texs = vec![
            texture([0., 1.]),
            texture([1., 1.]),
            texture([1., 0.]),
            texture([0., 0.]),
        ];

        let mut idx = 0;
        let mut tex_vert = vec![];

        // TODO: use VecDequeue
        self.vertex.reverse();
        while let Some(vert) = self.vertex.pop() {
            let tex = *texs.get(idx).expect("`texs` length is incorrect");
            tex_vert.push(texture_vertex(vert, tex));
            idx += 1;
        }

        self.texture = Some(tex_vert);
        self.texture_descriptor = Some(descriptor);

        Mesh::Texture(Box::new(self))
    }
}
