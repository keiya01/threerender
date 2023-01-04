use std::rc::Rc;

use crate::{
    mesh::{primitive::Primitive, MeshType},
    unit::{Position, RGBA}, RendererState,
};

pub struct EntityDescriptor {
    pub id: String,
    pub mesh: Rc<dyn Primitive>,
    pub fill_color: RGBA,
    // TODO: use human readable unit
    pub coordinates: Position,
    pub state: EntityRendererState,
}

#[derive(Debug)]
pub struct Entity {
    pub id: String,
    pub fill_color: RGBA,
    pub coordinates: Position,
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
}

impl EntityRendererState {
    pub fn from_renderer_state(state: RendererState) -> Self {
        Self {
            mesh_type: state.mesh_type
        }
    }
}

impl Eq for EntityRendererState {}
