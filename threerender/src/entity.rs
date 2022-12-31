use std::rc::Rc;

use glam::Mat4;

use crate::{mesh::primitive::Primitive, unit::RGBA};

pub struct EntityDescriptor {
    pub mesh: Rc<dyn Primitive>,
    pub fill_color: RGBA,
    // TODO: use human readable unit
    pub coordinates: Mat4,
}

#[derive(Debug)]
pub struct Entity {
    pub(super) fill_color: RGBA,
    pub(super) coordinates: Mat4,
}

impl Entity {
    pub fn coordinates_mut(&mut self) -> &mut Mat4 {
        &mut self.coordinates
    }
}
