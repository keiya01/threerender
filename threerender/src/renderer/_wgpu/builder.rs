use wgpu::Features;

use crate::RendererBuilder;

// TODO: optimize entity buffer size
const MAXIMUM_ENTITY_LENGTH: u64 = 1048576; // 1MB

pub struct RendererSpecificAttributes {
    pub(super) maximum_entity_length: u64,
    pub(super) features: Features,
    pub(super) adapter_features: bool,
}

impl Default for RendererSpecificAttributes {
    fn default() -> Self {
        Self {
            maximum_entity_length: MAXIMUM_ENTITY_LENGTH,
            features: Features::empty(),
            adapter_features: false,
        }
    }
}

pub trait WGPURendererBuilder {
    fn set_maximum_entity_length(&mut self, maximum_entity_length: u64);
    fn set_features(&mut self, features: Features);
    fn set_adapter_features(&mut self, enable: bool);
}

impl WGPURendererBuilder for RendererBuilder {
    fn set_maximum_entity_length(&mut self, maximum_entity_length: u64) {
        self.renderer_specific_attributes.maximum_entity_length = maximum_entity_length;
    }
    fn set_features(&mut self, features: Features) {
        self.renderer_specific_attributes.features = features;
    }
    fn set_adapter_features(&mut self, enable: bool) {
        self.renderer_specific_attributes.adapter_features = enable;
    }
}
