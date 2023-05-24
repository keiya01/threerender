use std::rc::Rc;

use getset::{Getters, MutGetters, Setters};
use threerender_color::rgb::RGBA;
use threerender_math::Transform;

use crate::mesh::{Mesh, MeshType, PolygonMode, Topology};

/// A descriptor to setup an entity to the renderer.
#[derive(Debug, Clone)]
pub struct EntityDescriptor {
    pub id: String,
    pub mesh: Option<Rc<Mesh>>,
    pub fill_color: RGBA,
    pub transform: Transform,
    pub reflection: ReflectionStyle,
    pub children: Vec<EntityDescriptor>,
    pub state: EntityRendererState,
}

impl Default for EntityDescriptor {
    fn default() -> Self {
        EntityDescriptor {
            id: "".to_string(),
            mesh: None,
            fill_color: RGBA::default(),
            transform: Default::default(),
            reflection: ReflectionStyle::default(),
            children: vec![],
            state: EntityRendererState::default(),
        }
    }
}

impl EntityDescriptor {
    pub fn flatten_mesh_length(&self) -> usize {
        let v = match self.mesh {
            Some(_) => 1,
            None => 0,
        };
        v + Self::recursive_mesh_length(&self.children)
    }

    pub fn recursive_mesh_length(children: &Vec<EntityDescriptor>) -> usize {
        let mut cnt = 0;
        for child in children {
            if child.mesh.is_some() {
                cnt += 1
            }
            cnt += Self::recursive_mesh_length(&child.children);
        }
        cnt
    }

    pub fn infer_mesh_type(&mut self) {
        match self.state.mesh_type {
            Some(_) => {}
            None => {
                self.state.mesh_type = match self.mesh.as_deref() {
                    Some(Mesh::Entity(_)) => Some(MeshType::Entity),
                    Some(Mesh::Texture(_)) => Some(MeshType::Texture),
                    _ => None,
                };
            }
        };
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct RendererState {
    pub mesh_type: MeshType,
    pub topology: Topology,
    pub polygon_mode: PolygonMode,
}

#[derive(Hash, Default, PartialEq, Debug, Clone)]
pub struct EntityRendererState {
    pub topology: Topology,
    pub polygon_mode: PolygonMode,
    pub mesh_type: Option<MeshType>,
}

impl EntityRendererState {
    pub fn from_renderer_state(state: RendererState) -> Self {
        Self {
            topology: state.topology,
            polygon_mode: state.polygon_mode,
            mesh_type: Some(state.mesh_type),
        }
    }
}

impl Eq for EntityRendererState {}

#[derive(Debug, Clone, Getters, MutGetters, Setters)]
pub struct ReflectionStyle {
    pub brightness: f32,
    pub shininess: f32,
    pub specular: f32,
}

impl Default for ReflectionStyle {
    fn default() -> Self {
        Self {
            brightness: 0.,
            shininess: 0.,
            specular: 1.,
        }
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use threerender_color::rgb::RGBA;

    use crate::mesh::{DefaultMesh, EntityMesh};

    use super::EntityDescriptor;

    #[derive(Debug)]
    struct Entity;
    impl EntityMesh for Entity {
        fn vertex(&self) -> &[crate::mesh::Vertex] {
            &[]
        }

        fn index(&self) -> Option<&[u16]> {
            None
        }
    }

    #[test]
    fn test_recursive_count() {
        let mut descriptor = EntityDescriptor {
            id: "".to_string(),
            mesh: Some(Rc::new(DefaultMesh.use_entity())),
            fill_color: RGBA::default(),
            transform: Default::default(),
            reflection: super::ReflectionStyle::default(),
            children: vec![],
            state: super::EntityRendererState::default(),
        };
        let mut descriptor_no_mesh = EntityDescriptor {
            id: "".to_string(),
            mesh: None,
            fill_color: RGBA::default(),
            transform: Default::default(),
            reflection: super::ReflectionStyle::default(),
            children: vec![],
            state: super::EntityRendererState::default(),
        };
        descriptor.children.push(descriptor.clone());
        descriptor_no_mesh.children.push(descriptor.clone());
        descriptor.children.push(descriptor_no_mesh);

        assert_eq!(descriptor.flatten_mesh_length(), 4);
    }
}