use std::{rc::Rc, mem};

use anyhow::Result;
use gltf::{mesh::util::{ReadIndices, ReadTexCoords}};
use threerender_color::rgb::RGBA;
use threerender_math::Transform;
use threerender_traits::{
    entity::{EntityDescriptor, EntityRendererState, ReflectionStyle},
    mesh::{vertex, EntityMesh, Vertex, TextureMesh, TextureVertex, TextureFormat, texture_vertex, Mesh, MeshType},
};

use super::{
    err::GltfError,
    fetcher::{Buffer, GltfFetcher},
};

#[derive(Debug, Clone)]
pub struct GltfMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Option<Vec<u16>>,
    pub textures: Option<Vec<TextureVertex>>,
    pub tex_coords: Option<Vec<[f32; 2]>>,
    pub material: Option<Material>,
}

impl GltfMesh {
    fn new() -> Self {
        Self {
            vertices: vec![],
            indices: None,
            tex_coords: None,
            textures: None,
            material: None,
        }
    }
}

impl EntityMesh for GltfMesh {
    fn vertex(&self) -> &[Vertex] {
        &self.vertices
    }

    fn index(&self) -> Option<&[u16]> {
        match &self.indices {
            Some(idx) => Some(idx),
            None => None,
        }
    }
}

impl TextureMesh for GltfMesh {
    fn texture(&self) -> Option<&[TextureVertex]> {
        self.textures.as_ref().map(|t| &t[..])
    }
    fn width(&self) -> u32 {
        self.material.as_ref().unwrap().base_color_texture.as_ref().unwrap().size.0
    }
    fn height(&self) -> u32 {
        self.material.as_ref().unwrap().base_color_texture.as_ref().unwrap().size.1
    }
    fn format(&self) -> &TextureFormat {
        &self.material.as_ref().unwrap().base_color_texture.as_ref().unwrap().format
    }
    fn data(&self) -> &[u8] {
        &self.material.as_ref().unwrap().base_color_texture.as_ref().unwrap().data
    }
    fn use_texture(mut self) -> Mesh {
        let mut tex_vert = vec![];

        let verts = mem::take(&mut self.vertices);
        for (idx, vert) in verts.into_iter().enumerate() {
            let tex = *self.tex_coords.as_ref().unwrap().get(idx).expect("`texs` length is incorrect");
            tex_vert.push(texture_vertex(vert, tex));
        }

        self.textures = Some(tex_vert);

        Mesh::Texture(Box::new(self))
    }
}

pub struct GltfLoader {
    pub entities: Vec<EntityDescriptor>,
}

impl GltfLoader {
    pub fn from_byte<F>(name: &str, bytes: &[u8], fetcher: F) -> Result<Self, GltfError>
    where
        F: GltfFetcher,
    {
        Self::load(name, gltf::Gltf::from_slice(bytes)?, fetcher)
    }

    // TODO: Support animation, material, skin, camera and so on.
    fn load<F>(name: &str, data: gltf::Gltf, mut fetcher: F) -> Result<Self, GltfError>
    where
        F: GltfFetcher,
    {
        let buffers = Self::load_buffers(&data, &fetcher)?;
        let mut temp_meshes = vec![];

        let mut materials = vec![];
        for material in data.materials() {
            materials.push(Material::from_material(&material, &mut fetcher)?);
        }

        for mesh in data.meshes() {
            let mut entity = GltfMesh::new();

            for prim in mesh.primitives() {
                let reader = prim.reader(|b| buffers.get(b.index()).map(|v| &v[..]));

                let positions = reader.read_positions();
                let normals = reader.read_normals();

                if positions.as_ref().map(|t| t.len()) != normals.as_ref().map(|n| n.len()) {
                    // FIXME(@keiya01): Cover this case
                    unimplemented!(
                        "Length of positions is different with normals: {:?}, {:?}",
                        positions,
                        normals
                    );
                }

                if let Some(positions) = positions {
                    // FIXME(@keiya01): Default normal should be fixed
                    let _: Vec<_> = positions
                        .map(|p| {
                            entity
                                .vertices
                                .push(vertex([p[0], p[1], p[2], 1.], [1.; 3]))
                        })
                        .collect();
                }
                if let Some(normals) = normals {
                    let _: Vec<_> = normals
                        .enumerate()
                        .map(|(i, n)| entity.vertices.get_mut(i).map(|v| v.normal = n))
                        .collect();
                }

                if let Some(indices) = reader.read_indices() {
                    match indices {
                        ReadIndices::U8(indices) => {
                            let ei = &mut entity.indices;
                            for idx in indices {
                                match ei {
                                    Some(ref mut ei) => ei.push(idx as u16),
                                    None => *ei = Some(vec![idx as u16]),
                                }
                            }
                        }
                        ReadIndices::U16(indices) => {
                            let ei = &mut entity.indices;
                            for idx in indices {
                                match ei {
                                    Some(ref mut ei) => ei.push(idx),
                                    None => *ei = Some(vec![idx]),
                                }
                            }
                        }
                        ReadIndices::U32(indices) => {
                            let ei = &mut entity.indices;
                            for idx in indices {
                                match ei {
                                    Some(ref mut ei) => ei.push(idx as u16),
                                    None => *ei = Some(vec![idx as u16]),
                                }
                            }
                        }
                    }
                }

                for tex in data.textures() {
                    if let Some(tex_coords) = reader.read_tex_coords(tex.index() as u32) {
                        match tex_coords {
                            ReadTexCoords::U8(coords) => {
                                let et = &mut entity.tex_coords;
                                for coord in coords {
                                    let coord = [coord[0] as f32, coord[1] as f32];
                                    match et {
                                        Some(ref mut et) => et.push(coord),
                                        None => *et = Some(vec![coord]),
                                    }
                                }
                            }
                            ReadTexCoords::U16(coords) => {
                                let et = &mut entity.tex_coords;
                                for coord in coords {
                                    let coord = [coord[0] as f32, coord[1] as f32];
                                    match et {
                                        Some(ref mut et) => et.push(coord),
                                        None => *et = Some(vec![coord]),
                                    }
                                }
                            }
                            ReadTexCoords::F32(coords) => {
                                let et = &mut entity.tex_coords;
                                for coord in coords {
                                    let coord = [coord[0] as f32, coord[1] as f32];
                                    match et {
                                        Some(ref mut et) => et.push(coord),
                                        None => *et = Some(vec![coord]),
                                    }
                                }
                            }
                        }
                    }
                }

                // FIXME(@keiya01): Handle duplicated material
                entity.material = prim
                    .material()
                    .index()
                    .and_then(|i| materials.get(i).cloned());
            }

            temp_meshes.push(entity);
        }

        // Flatting glTF children of node with mesh index.
        fn search_node<F>(nodes: Vec<gltf::Node>, f: &F) -> Vec<EntityDescriptor>
        where
            F: Fn(&gltf::Node, Vec<EntityDescriptor>) -> EntityDescriptor,
        {
            let mut entities = vec![];
            for node in nodes {
                entities.push(f(&node, search_node(node.children().collect(), f)));
            }
            entities
        }

        let f = |node: &gltf::Node, children: Vec<EntityDescriptor>| {
            let mesh = node.mesh();
            let node_idx = node.index();
            let node = GltfNode::from_node(node);
            match mesh {
                Some(mesh) => {
                    let mesh_idx = mesh.index();
                    let mesh = temp_meshes
                        .get(mesh_idx)
                        .expect("Mesh length hos to match with node index");
                    let mesh = mesh.clone();
                    #[allow(unused_parens)]
                    let (color) = mesh
                        .material
                        .as_ref()
                        .map_or_else(Default::default, |m| (m.base_color));
                    let is_tex = mesh.tex_coords.is_some();
                    EntityDescriptor {
                        id: format!("{name}:{node_idx}"),
                        mesh: Some(Rc::new(match mesh.tex_coords {
                            Some(ref coords) if !coords.is_empty() => mesh.use_texture(),
                            _ => mesh.use_entity(),
                        })),
                        fill_color: RGBA::from_f32(color[0], color[1], color[2], color[3]),
                        transform: node.local_transform,
                        reflection: ReflectionStyle::default(),
                        children,
                        state: EntityRendererState {
                            mesh_type: Some(if is_tex { MeshType::Texture } else { MeshType::Entity }),
                            ..Default::default()
                        },
                    }
                }
                None => EntityDescriptor {
                    id: format!("{name}:{node_idx}"),
                    mesh: None,
                    fill_color: RGBA::default(),
                    transform: node.local_transform,
                    reflection: ReflectionStyle::default(),
                    children,
                    state: EntityRendererState::default(),
                },
            }
        };

        let mut entities = vec![];
        for scene in data.scenes() {
            entities.push(EntityDescriptor {
                id: format!("{name}:scene:{}", scene.index()),
                mesh: None,
                fill_color: RGBA::default(),
                transform: Transform::default(),
                reflection: ReflectionStyle::default(),
                // FIXME(@keiya01): Handle camera transform
                children: search_node(scene.nodes().collect(), &f),
                state: EntityRendererState::default(),
            });
        }

        Ok(Self { entities })
    }

    fn load_buffers<F>(data: &gltf::Gltf, fetcher: &F) -> Result<Vec<Buffer>, GltfError>
    where
        F: GltfFetcher,
    {
        let mut buffers = Vec::new();

        for buf in data.buffers() {
            match buf.source() {
                gltf::buffer::Source::Uri(uri) => {
                    if check_if_data_uri(uri) {
                        buffers.push(fetcher.parse_data_url(uri)?);
                    } else {
                        buffers.push(fetcher.fetch(uri)?);
                    }
                }
                gltf::buffer::Source::Bin => {
                    if let Some(blob) = data.blob.as_deref() {
                        buffers.push(blob.into());
                    } else {
                        return Err(GltfError::MissingBlob);
                    }
                }
            }
        }
        Ok(buffers)
    }
}

#[derive(Debug, Clone)]
pub struct MaterialTextureDescriptor {
    data: Rc<Vec<u8>>,
    size: (u32, u32),
    format: TextureFormat,
}

#[derive(Debug, Clone)]
pub struct Material {
    pub base_color: [f32; 4],
    pub base_color_texture: Option<MaterialTextureDescriptor>,
}

impl Material {
    fn from_material<F>(material: &gltf::Material, fetcher: &mut F) -> Result<Self, GltfError>
    where
        F: GltfFetcher,
    {
        let pbr = material.pbr_metallic_roughness();
        let color = pbr.base_color_factor();

        let base_color_texture = match pbr.base_color_texture() {
            Some(v) => {
                let data = match v.texture().source().source() {
                    gltf::image::Source::View { view, mime_type: _mime_type } => {
                        match view.buffer().source() {
                            gltf::buffer::Source::Uri(uri) => {
                                if check_if_data_uri(uri) {
                                    fetcher.parse_data_url(uri)?
                                } else {
                                    fetcher.fetch(uri)?
                                }
                            }
                            gltf::buffer::Source::Bin => {
                                unimplemented!()
                            }
                        }
                    },
                    gltf::image::Source::Uri { uri, mime_type: _mime_type } => {
                        fetcher.fetch(uri)?
                    }
                };
                let mut img = fetcher.load_image(data)?;
                Some(MaterialTextureDescriptor { data: Rc::new(img.data()), size: (img.width(), img.height()), format: img.format() })
            },
            None => None,
        };

        Ok(Self {
            base_color: color,
            base_color_texture,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct GltfNode {
    pub local_transform: Transform,
}

impl GltfNode {
    fn from_node(node: &gltf::Node) -> Self {
        let trs = match node.transform() {
            gltf::scene::Transform::Matrix { matrix } => Transform::from_cols_array_2d(matrix),
            gltf::scene::Transform::Decomposed {
                translation,
                rotation,
                scale,
            } => Transform::from_translation_rotation_scale_array(translation, rotation, scale),
        };

        Self {
            local_transform: trs,
        }
    }
}

fn check_if_data_uri(uri: &str) -> bool {
    uri.starts_with("data:")
}

#[cfg(test)]
#[test]
fn test_check_if_data_uri() {
    assert!(check_if_data_uri(
        "data:text/plain;base64,SGVsbG8sIFdvcmxkIQ=="
    ));
    assert!(!check_if_data_uri("https://example.com"));
}
