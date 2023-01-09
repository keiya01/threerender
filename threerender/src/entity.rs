use std::{rc::Rc};

use glam::Vec3;

use crate::{
    mesh::{traits::Mesh, MeshType, PolygonMode, Topology},
    unit::RGBA,
    RendererState,
};

pub struct EntityDescriptor {
    pub id: String,
    pub mesh: Rc<Mesh>,
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
    pub topology: Topology,
    pub polygon_mode: PolygonMode,
    pub mesh_type: MeshType,
}

impl EntityRendererState {
    pub fn from_renderer_state(state: RendererState) -> Self {
        Self {
            topology: state.topology,
            polygon_mode: state.polygon_mode,
            mesh_type: state.mesh_type,
        }
    }
}

impl Eq for EntityRendererState {}
