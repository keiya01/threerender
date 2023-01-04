use std::rc::Rc;

use crate::{
    mesh::primitive::Primitive,
    unit::{Position, RGBA},
};

pub struct EntityDescriptor {
    pub id: String,
    pub mesh: Rc<dyn Primitive>,
    pub fill_color: RGBA,
    // TODO: use human readable unit
    pub coordinates: Position,
}

#[derive(Debug)]
pub struct Entity {
    pub id: String,
    pub fill_color: RGBA,
    pub coordinates: Position,
}

pub trait EntityList {
    fn push(&mut self, descriptor: EntityDescriptor);
    fn items(&self) -> &[Entity];
    fn items_mut(&mut self) -> &mut [Entity];
}
