use std::f32::consts::PI;

use super::{
    traits::{EntityMesh, Mesh, TextureMesh},
    util::{texture, texture_vertex, vertex, TextureVertex, Vertex},
    TextureDescriptor, TextureFormat,
};

pub struct Square {
    vertex: Vec<Vertex>,
    index: Vec<u16>,

    texture_descriptor: Option<TextureDescriptor>,
    texture: Option<Vec<TextureVertex>>,
}

impl Square {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Square {
    fn default() -> Self {
        Self {
            vertex: vec![
                // Left
                vertex([-1., -1., -1., 1.], [-1., 0., 0.]),
                vertex([-1., -1., 1., 1.], [-1., 0., 0.]),
                vertex([-1., 1., 1., 1.], [-1., 0., 0.]),
                vertex([-1., 1., -1., 1.], [-1., 0., 0.]),
                // Back
                vertex([1., -1., -1., 1.], [0., 0., -1.]),
                vertex([-1., -1., -1., 1.], [0., 0., -1.]),
                vertex([-1., 1., -1., 1.], [0., 0., -1.]),
                vertex([1., 1., -1., 1.], [0., 0., -1.]),
                // Bottom
                vertex([-1., -1., -1., 1.], [0., -1., 0.]),
                vertex([1., -1., -1., 1.], [0., -1., 0.]),
                vertex([1., -1., 1., 1.], [0., -1., 0.]),
                vertex([-1., -1., 1., 1.], [0., -1., 0.]),
                // Right
                vertex([1., -1., 1., 1.], [1., 0., 0.]),
                vertex([1., -1., -1., 1.], [1., 0., 0.]),
                vertex([1., 1., -1., 1.], [1., 0., 0.]),
                vertex([1., 1., 1., 1.], [1., 0., 0.]),
                // Top
                vertex([-1., 1., -1., 1.], [0., 1., 0.]),
                vertex([-1., 1., 1., 1.], [0., 1., 0.]),
                vertex([1., 1., 1., 1.], [0., 1., 0.]),
                vertex([1., 1., -1., 1.], [0., 1., 0.]),
                // Front
                vertex([-1., -1., 1., 1.], [0., 0., 1.]),
                vertex([1., -1., 1., 1.], [0., 0., 1.]),
                vertex([1., 1., 1., 1.], [0., 0., 1.]),
                vertex([-1., 1., 1., 1.], [0., 0., 1.]),
            ],
            #[rustfmt::skip]
            index: vec![
                // Left
                0, 1, 2, 0, 2, 3,
                // Back
                4, 5, 6, 4, 6, 7,
                // Bottom
                8, 9, 10, 8, 10, 11,
                // Right
                12, 13, 14, 12, 14, 15,
                // Top
                16, 17, 18, 16, 18, 19,
                // Front
                20, 21, 22, 20, 22, 23,
            ],
            texture: None,
            texture_descriptor: None,
        }
    }
}

impl EntityMesh for Square {
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

impl TextureMesh for Square {
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
            let tex = *texs.get(idx % 4).expect("`texs` length is incorrect");
            tex_vert.push(texture_vertex(vert, tex));
            idx += 1;
        }

        self.texture = Some(tex_vert);
        self.texture_descriptor = Some(descriptor);

        Mesh::Texture(Box::new(self))
    }
}

#[derive(Default)]
pub struct Sphere {
    vertex: Option<Vec<Vertex>>,
    index: Option<Vec<u16>>,

    slices: u16,
    stacks: u16,

    texture_descriptor: Option<TextureDescriptor>,
    texture: Option<Vec<TextureVertex>>,
}

impl Sphere {
    pub fn new(slices: u16, stacks: u16) -> Self {
        Self {
            vertex: None,
            index: None,
            texture_descriptor: None,
            texture: None,
            slices,
            stacks,
        }
    }

    fn make_data(&self) -> (Vec<Vertex>, Vec<u16>) {
        let slices = self.slices;
        let stacks = self.stacks;

        let mut vertices = vec![];
        let mut indices = vec![];
        for j in 0..stacks + 1 {
            let t = (j as f32) / (stacks as f32);
            let y = (PI * t).cos();
            let r = (PI * t).sin();

            // For indices
            let k = (slices + 1) * j;

            for i in 0..slices + 1 {
                // Make vertices
                let s = (i as f32) / (slices as f32);
                let z = r * (2. * PI * s).cos();
                let x = r * (2. * PI * s).sin();
                vertices.push(vertex([x, y, z, 1.], [x, y, z]));

                if j < stacks && i < slices {
                    // Make indices
                    let k0 = k + i;
                    let k1 = k0 + 1;
                    let k2 = k1 + slices;
                    let k3 = k2 + 1;

                    indices.push(k0);
                    indices.push(k2);
                    indices.push(k3);

                    indices.push(k0);
                    indices.push(k3);
                    indices.push(k1);
                }
            }
        }
        (vertices, indices)
    }

    fn make_texture_data(&self) -> (Vec<TextureVertex>, Vec<u16>) {
        let slices = self.slices;
        let stacks = self.stacks;

        let mut vertices = vec![];
        let mut indices = vec![];

        for j in 0..stacks + 1 {
            let t = (j as f32) / (stacks as f32);
            let y = (PI * t).cos();
            let r = (PI * t).sin();

            // For indices
            let k = (slices + 1) * j;

            let ty = (j as f32) / (stacks as f32);

            for i in 0..slices + 1 {
                // Make vertices
                let s = (i as f32) / (slices as f32);
                let z = r * (2. * PI * s).cos();
                let x = r * (2. * PI * s).sin();

                let ver = vertex([x, y, z, 1.], [x, y, z]);
                let tex = texture([(i as f32 / slices as f32), ty]);

                vertices.push(texture_vertex(ver, tex));

                if j < stacks && i < slices {
                    // Make indices
                    let k0 = k + i;
                    let k1 = k0 + 1;
                    let k2 = k1 + slices;
                    let k3 = k2 + 1;

                    indices.push(k0);
                    indices.push(k2);
                    indices.push(k3);

                    indices.push(k0);
                    indices.push(k3);
                    indices.push(k1);
                }
            }
        }
        (vertices, indices)
    }
}

impl EntityMesh for Sphere {
    fn vertex(&self) -> &[Vertex] {
        self.vertex
            .as_ref()
            .expect("You should invoke `use_entity` or `use_texture`.")
    }

    fn index(&self) -> Option<&[u16]> {
        self.index.as_ref().map(|v| &v[..])
    }

    fn use_entity(mut self) -> Mesh {
        let (vertex, index) = self.make_data();
        self.vertex = Some(vertex);
        self.index = Some(index);
        Mesh::Entity(Box::new(self))
    }
}

impl TextureMesh for Sphere {
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
        let (vertex, index) = self.make_texture_data();
        self.texture = Some(vertex);
        self.index = Some(index);
        self.texture_descriptor = Some(descriptor);
        Mesh::Texture(Box::new(self))
    }
}
