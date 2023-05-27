use std::rc::Rc;

use threerender_traits::mesh::{
    texture, texture_vertex, vertex, EntityMesh, Mesh, TextureFormat, TextureMesh, TextureVertex,
    Vertex,
};

use crate::TextureDescriptor;

#[derive(Debug)]
pub struct Square {
    vertex: Vec<Vertex>,
    index: Vec<u16>,

    texture_descriptor: Option<TextureDescriptor>,
    texture: Option<Vec<TextureVertex>>,
}

impl Square {
    pub fn new(texture_descriptor: Option<TextureDescriptor>) -> Self {
        Self {
            texture_descriptor,
            ..Self::default()
        }
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
        Mesh::Entity(Rc::new(self))
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

    fn use_texture(mut self) -> Mesh {
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

        Mesh::Texture(Rc::new(self))
    }
}
