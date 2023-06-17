use std::rc::Rc;

use getset::{Getters, MutGetters, Setters};
use threerender_color::rgb::RGBA;
use threerender_math::Transform;

use crate::{
    image::Image,
    mesh::{Mesh, PolygonMode, Topology},
};

/// A descriptor to setup an entity to the renderer.
#[derive(Debug, Clone)]
pub struct EntityDescriptor {
    pub id: String,
    pub mesh: Option<Rc<dyn Mesh>>,
    pub fill_color: RGBA,
    pub transform: Transform,
    pub reflection: ReflectionStyle,
    pub children: Vec<EntityDescriptor>,
    pub state: EntityRendererState,
    pub texture: Option<Rc<dyn Image>>,
    pub normal_map: Option<Rc<dyn Image>>,
    pub receive_shadow: bool,
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
            texture: None,
            normal_map: None,
            receive_shadow: true,
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
}

#[derive(Clone, Copy, Debug)]
pub struct RendererState {
    pub topology: Topology,
    pub polygon_mode: PolygonMode,
}

impl Default for RendererState {
    fn default() -> Self {
        Self {
            topology: Default::default(),
            polygon_mode: Default::default(),
        }
    }
}

#[derive(Hash, PartialEq, Debug, Clone)]
pub struct EntityRendererState {
    pub topology: Topology,
    pub polygon_mode: PolygonMode,
}

impl Default for EntityRendererState {
    fn default() -> Self {
        Self {
            topology: Default::default(),
            polygon_mode: Default::default(),
        }
    }
}

impl EntityRendererState {
    pub fn from_renderer_state(state: RendererState) -> Self {
        Self {
            topology: state.topology,
            polygon_mode: state.polygon_mode,
        }
    }
}

impl Eq for EntityRendererState {}

#[derive(Debug, Clone, Getters, MutGetters, Setters)]
pub struct ReflectionStyle {
    pub intensity: f32,
    pub specular: f32,
}

impl Default for ReflectionStyle {
    fn default() -> Self {
        Self {
            intensity: 0.,
            specular: 0.,
        }
    }
}

#[cfg(test)]
mod test {
    use std::{cell::RefCell, rc::Rc};

    use threerender_color::rgb::RGBA;

    use crate::mesh::{DefaultMesh, Mesh, Vertex};

    use super::EntityDescriptor;

    #[derive(Debug)]
    struct Entity;
    impl Mesh for Entity {
        fn vertex(&self) -> Rc<RefCell<Vec<Vertex>>> {
            Rc::new(RefCell::new(vec![]))
        }

        fn index(&self) -> Option<&[u16]> {
            None
        }
    }

    #[test]
    fn test_recursive_count() {
        let mut descriptor = EntityDescriptor {
            id: "".to_string(),
            mesh: Some(Rc::new(DefaultMesh)),
            fill_color: RGBA::default(),
            transform: Default::default(),
            reflection: super::ReflectionStyle::default(),
            children: vec![],
            state: super::EntityRendererState::default(),
            texture: None,
            normal_map: None,
            receive_shadow: true,
        };
        let mut descriptor_no_mesh = EntityDescriptor {
            id: "".to_string(),
            mesh: None,
            fill_color: RGBA::default(),
            transform: Default::default(),
            reflection: super::ReflectionStyle::default(),
            children: vec![],
            state: super::EntityRendererState::default(),
            texture: None,
            normal_map: None,
            receive_shadow: true,
        };
        descriptor.children.push(descriptor.clone());
        descriptor_no_mesh.children.push(descriptor.clone());
        descriptor.children.push(descriptor_no_mesh);

        assert_eq!(descriptor.flatten_mesh_length(), 4);
    }
}
