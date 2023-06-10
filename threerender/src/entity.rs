use threerender_color::rgb::RGBA;
use threerender_math::{
    trs::{Rotation, Scale, Translation},
    Quat, Transform, Vec3,
};
use threerender_traits::entity::{EntityDescriptor, EntityRendererState, ReflectionStyle};

/// An entity to render actually.
#[derive(Debug, Default, Clone)]
pub struct Entity {
    pub id: String,
    pub fill_color: RGBA,
    pub transform: Transform,
    pub reflection: ReflectionStyle,
    pub children: Vec<Entity>,
    pub state: EntityRendererState,
    pub normal_map_idx: Option<i32>,
    pub tex_idx: Option<i32>,
}

impl Translation for Entity {
    fn translation(&self) -> &Vec3 {
        &self.transform.translation
    }
    fn translation_mut(&mut self) -> &mut Vec3 {
        &mut self.transform.translation
    }
}

impl Rotation for Entity {
    fn rotation(&self) -> &Quat {
        &self.transform.rotation
    }
    fn rotation_mut(&mut self) -> &mut Quat {
        &mut self.transform.rotation
    }
}

impl Scale for Entity {
    fn scale(&self) -> &Vec3 {
        &self.transform.scale
    }
    fn scale_mut(&mut self) -> &mut Vec3 {
        &mut self.transform.scale
    }
}

pub trait EntityList {
    fn push(&mut self, descriptor: EntityDescriptor);
    fn items(&self) -> &[Entity];
    fn items_mut(&mut self) -> &mut [Entity];
}
