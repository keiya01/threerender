use std::rc::Rc;

use anyhow::Result;
use gltf::mesh::util::ReadIndices;
use threerender_color::rgb::RGBA;
use threerender_math::Transform;
use threerender_traits::{
    entity::{EntityDescriptor, EntityRendererState, ReflectionStyle},
    mesh::{vertex, EntityMesh, Mesh, Vertex},
};

use super::{
    err::GltfError,
    fetcher::{Buffer, GltfFetcher},
};

// TODO: Support texture
#[derive(Debug, Clone)]
pub struct GltfMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Option<Vec<u16>>,
    pub material: Option<Material>,
}

impl GltfMesh {
    fn new() -> Self {
        Self {
            vertices: vec![],
            indices: None,
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
    fn load<F>(name: &str, data: gltf::Gltf, fetcher: F) -> Result<Self, GltfError>
    where
        F: GltfFetcher,
    {
        let buffers = Self::load_buffers(&data, fetcher)?;
        let mut temp_meshes = vec![];

        let mut materials = vec![];
        for material in data.materials() {
            materials.push(Material::from_material(&material));
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

                // FIXME(@keiya01): Handle duplicated material
                entity.material = prim
                    .material()
                    .index()
                    .and_then(|i| materials.get(i).copied());
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
                        .map_or_else(Default::default, |m| (m.base_color));
                    EntityDescriptor {
                        id: format!("{name}:{node_idx}"),
                        // FIXME(@keiya01): Check texture
                        mesh: Some(Rc::new(Mesh::Entity(Box::new(mesh)))),
                        fill_color: RGBA::from_f32(color[0], color[1], color[2], color[3]),
                        transform: node.local_transform,
                        reflection: ReflectionStyle::default(),
                        children,
                        state: EntityRendererState::default(),
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

    fn load_buffers<F>(data: &gltf::Gltf, fetcher: F) -> Result<Vec<Buffer>, GltfError>
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

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub base_color: [f32; 4],
}

impl Material {
    fn from_material(material: &gltf::Material) -> Self {
        let pbr = material.pbr_metallic_roughness();
        let color = pbr.base_color_factor();

        Self { base_color: color }
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
