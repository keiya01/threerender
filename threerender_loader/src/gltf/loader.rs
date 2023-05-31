use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use gltf::mesh::util::{ReadIndices, ReadTexCoords};
use threerender_color::rgb::RGBA;
use threerender_math::Transform;
use threerender_traits::{
    entity::{EntityDescriptor, EntityRendererState, ReflectionStyle},
    image::{DefaultImage, Image},
    mesh::{vertex, Mesh, Vertex},
    types::Buffer,
};

use super::{err::GltfError, fetcher::GltfFetcher, GltfHandler};

#[derive(Debug, Clone)]
pub struct GltfMesh {
    pub vertices: Rc<RefCell<Vec<Vertex>>>,
    pub indices: Option<Vec<u16>>,
    pub tex_coords: Option<Vec<[f32; 2]>>,
    pub material: Option<Material>,
}

impl GltfMesh {
    fn new() -> Self {
        Self {
            vertices: Rc::new(RefCell::new(vec![])),
            indices: None,
            tex_coords: None,
            material: None,
        }
    }

    fn prepare_textures(&mut self) {
        let tex_coords = match &self.tex_coords {
            Some(c) => c,
            None => return,
        };

        for (idx, vert) in self.vertices.borrow_mut().iter_mut().enumerate() {
            let tex = tex_coords.get(idx).expect("`texs` length is incorrect");
            vert.tex = *tex;
        }
    }
}

impl Mesh for GltfMesh {
    fn vertex(&self) -> Rc<RefCell<Vec<Vertex>>> {
        self.vertices.clone()
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
    pub fn from_byte<F, H>(
        name: &str,
        bytes: &[u8],
        fetcher: F,
        handler: H,
    ) -> Result<Self, GltfError>
    where
        F: GltfFetcher,
        H: GltfHandler,
    {
        Self::load(name, gltf::Gltf::from_slice(bytes)?, fetcher, handler)
    }

    // TODO: Support animation, material, skin, camera and so on.
    fn load<F, H>(
        name: &str,
        data: gltf::Gltf,
        mut fetcher: F,
        handler: H,
    ) -> Result<Self, GltfError>
    where
        F: GltfFetcher,
        H: GltfHandler,
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
                                .borrow_mut()
                                .push(vertex([p[0], p[1], p[2], 1.], [1.; 3]))
                        })
                        .collect();
                }
                if let Some(normals) = normals {
                    let _: Vec<_> = normals
                        .enumerate()
                        .map(|(i, n)| {
                            entity
                                .vertices
                                .borrow_mut()
                                .get_mut(i)
                                .map(|v| v.normal = n)
                        })
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
                                    let coord = [coord[0], coord[1]];
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

                entity.prepare_textures();
            }

            temp_meshes.push(Rc::new(entity));
        }

        // Flatting glTF children of node with mesh index.
        fn search_node<F>(nodes: Vec<gltf::Node>, f: &F) -> Result<Vec<EntityDescriptor>, GltfError>
        where
            F: Fn(&gltf::Node, Vec<EntityDescriptor>) -> Result<EntityDescriptor, GltfError>,
        {
            let mut entities = vec![];
            for node in nodes {
                entities.push(f(&node, search_node(node.children().collect(), f)?)?);
            }
            Ok(entities)
        }

        let f = |row_node: &gltf::Node,
                 children: Vec<EntityDescriptor>|
         -> Result<EntityDescriptor, GltfError> {
            let mesh = row_node.mesh();
            let node_idx = row_node.index();
            let node = GltfNode::from_node(row_node);
            match mesh {
                Some(mesh) => {
                    let mesh_idx = mesh.index();
                    let mesh = temp_meshes
                        .get(mesh_idx)
                        .expect("Mesh length hos to match with node index");
                    let mesh = mesh.clone();
                    #[allow(unused_parens)]
                    let (color, texture, normal_map) =
                        mesh.material.as_ref().map_or_else(Default::default, |m| {
                            (
                                m.base_color,
                                m.base_color_texture.as_ref().map(|b| b.image.clone()),
                                m.normal_map.clone(),
                            )
                        });

                    let normal_map = match normal_map {
                        Some(n) => Some(Rc::new(DefaultImage::from_buffer(&n)?) as Rc<dyn Image>),
                        None => None,
                    };

                    let mut desc = EntityDescriptor {
                        id: format!("{name}:{node_idx}"),
                        mesh: Some(mesh.clone()),
                        fill_color: RGBA::from_f32(color[0], color[1], color[2], color[3]),
                        transform: node.local_transform,
                        reflection: ReflectionStyle::default(),
                        children,
                        state: EntityRendererState::default(),
                        texture,
                        normal_map,
                    };
                    handler.on_create(&mut desc, Some(&mesh), row_node);
                    Ok(desc)
                }
                None => {
                    let mut desc = EntityDescriptor {
                        id: format!("{name}:{node_idx}"),
                        mesh: None,
                        fill_color: RGBA::default(),
                        transform: node.local_transform,
                        reflection: ReflectionStyle::default(),
                        children,
                        state: EntityRendererState::default(),
                        texture: None,
                        normal_map: None,
                    };
                    handler.on_create(&mut desc, None, row_node);
                    Ok(desc)
                }
            }
        };

        let mut entities = vec![];
        // FIXME(@keiya01): Handle camera transform
        for scene in data.scenes() {
            entities.push(EntityDescriptor {
                id: format!("{name}:scene:{}", scene.index()),
                mesh: None,
                fill_color: RGBA::default(),
                transform: Transform::default(),
                reflection: ReflectionStyle::default(),
                children: search_node(scene.nodes().collect(), &f)?,
                state: EntityRendererState::default(),
                texture: None,
                normal_map: None,
            });
            handler.after_root(&mut entities, &scene);
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
    image: Rc<dyn Image>,
}

#[derive(Debug, Clone)]
pub struct Material {
    pub base_color: [f32; 4],
    pub base_color_texture: Option<MaterialTextureDescriptor>,
    pub metalness: f32,
    pub roughness: f32,
    pub normal_map: Option<Buffer>,
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
                let data = get_buffer_from_source(&v.texture().source().source(), fetcher)?;
                let img = fetcher.load_image(data)?;
                Some(MaterialTextureDescriptor { image: img })
            }
            None => None,
        };

        let metalness = pbr.metallic_factor();
        let roughness = pbr.roughness_factor();

        let normal_map = match material.normal_texture() {
            Some(n) => Some(get_buffer_from_source(
                &n.texture().source().source(),
                fetcher,
            )?),
            None => None,
        };

        Ok(Self {
            base_color: color,
            base_color_texture,
            metalness,
            roughness,
            normal_map,
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

fn get_buffer_from_source<F>(source: &gltf::image::Source, fetcher: &F) -> Result<Buffer, GltfError>
where
    F: GltfFetcher,
{
    Ok(match source {
        gltf::image::Source::View {
            view,
            mime_type: _mime_type,
        } => match view.buffer().source() {
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
        },
        gltf::image::Source::Uri {
            uri,
            mime_type: _mime_type,
        } => fetcher.fetch(uri)?,
    })
}
