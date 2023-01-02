use std::rc::Rc;

use crate::{
    mesh::primitive::Primitive,
    unit::{Position, RGBA},
};

pub struct EntityDescriptor {
    pub mesh: Rc<dyn Primitive>,
    pub fill_color: RGBA,
    // TODO: use human readable unit
    pub coordinates: Position,
}

#[derive(Debug)]
pub struct Entity {
    pub fill_color: RGBA,
    pub coordinates: Position,
}
