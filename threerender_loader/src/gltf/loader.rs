use anyhow::Result;
use gltf::mesh::util::ReadIndices;
use threerender_traits::mesh::{vertex, EntityMesh, Vertex};

use super::{
    err::GltfError,
    fetcher::{Buffer, GltfFetcher},
};

// TODO: Support texture
#[derive(Debug)]
pub struct GltfEntity {
    vertices: Vec<Vertex>,
    indices: Option<Vec<u16>>,
}

impl GltfEntity {
    fn new() -> Self {
        Self {
            vertices: vec![],
            indices: None,
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
        let mut entities = vec![];

        for mesh in data.meshes() {
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

                let mut entity = GltfEntity::new();
                if let Some(positions) = positions {
                    // FIXME(@keiya01): Default normal should be fixed
                    let _: Vec<_> = positions.map(|p| entity.vertices.push(vertex([p[0], p[1], p[2], 1.], [1.; 3]))).collect();
                }
                if let Some(normals) = normals {
                    let _: Vec<_> = normals
                        .enumerate()
                        .map(|(i, n)| entity.vertices.get_mut(i).map(|v| v.normal = n)).collect();
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
                entities.push(entity);
            }
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
