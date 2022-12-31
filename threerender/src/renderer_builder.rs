use crate::entity::EntityDescriptor;

use super::scene::{LightStyle, SceneStyle};

pub struct RendererBuilder {
    pub(super) entities: Vec<EntityDescriptor>,
    pub(super) enable_forward_depth: bool,
    pub(super) scene: Option<SceneStyle>,
    pub(super) width: u32,
    pub(super) height: u32,
}

impl Default for RendererBuilder {
    fn default() -> Self {
        Self {
            entities: vec![],
            enable_forward_depth: true,
            scene: Some(Default::default()),
            width: 0,
            height: 0,
        }
    }
}

#[allow(unused)]
impl RendererBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_size(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ..Default::default()
        }
    }

    pub fn push(&mut self, descriptor: EntityDescriptor) {
        self.entities.push(descriptor);
    }

    pub fn set_enable_forward_depth(&mut self, enable: bool) {
        self.enable_forward_depth = enable;
    }

    pub fn set_light(&mut self, light: LightStyle) {
        self.scene
            .as_mut()
            .expect("RendererBuilder has been consumed")
            .light = light;
    }

    pub fn set_width(&mut self, width: u32) {
        self.width = width;
    }

    pub fn set_height(&mut self, height: u32) {
        self.height = height;
    }
}
