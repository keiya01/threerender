use anyhow::Result;
use gltf::mesh::util::ReadIndices;
use threerender_math::{Mat4, Quat, Vec3};
use threerender_traits::mesh::{vertex, EntityMesh, Vertex};

use super::{
    err::GltfError,
    fetcher::{Buffer, GltfFetcher},
};

// TODO: Support texture
#[derive(Debug, Clone)]
pub struct GltfEntity {
    pub vertices: Vec<Vertex>,
    pub indices: Option<Vec<u16>>,
    pub material: Option<Material>,
    pub transform: Transform,
}

impl GltfEntity {
    fn new() -> Self {
        Self {
            vertices: vec![],
            indices: None,
            material: None,
            transform: Transform::default(),
        }
    }
}

impl EntityMesh for GltfEntity {
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
    pub entities: Vec<GltfEntity>,
}

impl GltfLoader {
    pub fn from_byte<F>(bytes: &[u8], fetcher: F) -> Result<Self, GltfError>
    where
        F: GltfFetcher,
    {
        Ok(Self::load(gltf::Gltf::from_slice(bytes)?, fetcher)?)
    }

    // TODO: Support animation, material, skin, camera and so on.
    fn load<F>(data: gltf::Gltf, fetcher: F) -> Result<Self, GltfError>
    where
        F: GltfFetcher,
    {
        let buffers = Self::load_buffers(&data, fetcher)?;
        let mut mesh_entities = vec![];

        let mut materials = vec![];
        for material in data.materials() {
            materials.push(Material::from_material(&material));
        }

        for mesh in data.meshes() {
            let mut entity = GltfEntity::new();

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
                    .and_then(|i| materials.get(i).and_then(|v| Some(v.clone())));
            }

            mesh_entities.push(entity);
        }

        // Flatting glTF children of node with mesh index.
        fn search_node(
            nodes: &mut Vec<(usize, Node)>,
            node: gltf::Node,
            parent: Option<Node>,
        ) {
            let child_mesh = node.mesh();
            let children = node.children();
            let own_node = Node::from_node(node, parent);

            if let Some(child_mesh) = child_mesh {
                nodes.push((child_mesh.index(), own_node.clone()));
            }

            for child in children {
                search_node(nodes, child, Some(own_node.clone()));
            }
        }

        let mut nodes = vec![];
        for scene in data.scenes() {
            // FIXME(@keiya01): Handle camera transform
            for node in scene.nodes() {
                search_node(&mut nodes, node, None);
            }
        }

        let mut entities = vec![];
        for (mesh, node) in nodes {
            mesh_entities.get(mesh).map(|entity| {
                let mut entity = entity.clone();
                entity.transform = node.global_transform;
                entities.push(entity);
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
pub struct Node {
    pub local_transform: Transform,
    pub global_transform: Transform,
}

impl Node {
    fn from_node(node: gltf::Node, parent: Option<Node>) -> Self {
        let trs = match node.transform() {
            gltf::scene::Transform::Matrix { matrix } => Transform::from_cols_array_2d(matrix),
            gltf::scene::Transform::Decomposed {
                translation,
                rotation,
                scale,
            } => Transform::from_translation_rotation_scale(translation, rotation, scale),
        };

        let parent = parent.unwrap_or_default();

        Self {
            local_transform: trs.clone(),
            global_transform: parent.global_transform.mul(&trs),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::default(),
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    fn from_cols_array_2d(matrix: [[f32; 4]; 4]) -> Self {
        let mat = Mat4::from_cols_array_2d(&matrix);
        let trs = mat.to_scale_rotation_translation();
        // Convert glam to threerender's vector type
        Self {
            scale: Vec3::from_array(&trs.0.to_array()),
            rotation: Quat::from_array(trs.1.to_array()),
            translation: Vec3::from_array(&trs.2.to_array()),
        }
    }

    fn from_translation_rotation_scale(translation: [f32; 3], rotation: [f32; 4], scale: [f32; 3]) -> Self {
        Self {
            translation: Vec3::from_array(&translation),
            rotation: Quat::from_array(rotation),
            scale: Vec3::from_array(&scale),
        }
    }

    fn mul(&self, node: &Self) -> Self {
        let mut next = Self::default();
        next.translation = self.transform_point(node.translation);
        next.rotation = self.rotation.mul(node.rotation);
        next.scale = self.scale.mul(node.scale);
        next
    }

    fn transform_point(&self, mut point: Vec3) -> Vec3 {
        point = self.scale.mul(point);
        point = self.rotation.mul_vec3(point);
        point = self.translation.add(point);
        point
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
