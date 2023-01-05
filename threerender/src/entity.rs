use std::rc::Rc;

use glam::Vec3;

use crate::{
    mesh::{primitive::Primitive, MeshType, PolygonMode},
    unit::RGBA,
    RendererState,
};

pub struct EntityDescriptor {
    pub id: String,
    pub mesh: Rc<dyn Primitive>,
    pub fill_color: RGBA,
    pub position: Vec3,
    pub dimension: Vec3,
    pub rotation: Vec3,
    pub state: EntityRendererState,
}

#[derive(Debug)]
pub struct Entity {
    pub id: String,
    pub fill_color: RGBA,
    pub position: Vec3,
    pub dimension: Vec3,
    pub rotation: Vec3,
    pub(super) state: EntityRendererState,
}

pub trait EntityList {
    fn push(&mut self, descriptor: EntityDescriptor);
    fn items(&self) -> &[Entity];
    fn items_mut(&mut self) -> &mut [Entity];
}

#[derive(Hash, Default, PartialEq, Debug)]
pub struct EntityRendererState {
    pub mesh_type: MeshType,
    pub polygon_mode: PolygonMode,
}

impl EntityRendererState {
    pub fn from_renderer_state(state: RendererState) -> Self {
        Self {
            mesh_type: state.mesh_type,
            polygon_mode: state.polygon_mode,
        }
    }
}

impl Eq for EntityRendererState {}
