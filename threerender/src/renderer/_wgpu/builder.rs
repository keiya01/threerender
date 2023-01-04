use crate::RendererBuilder;

// TODO: optimize entity buffer size
const MAXIMUM_ENTITY_LENGTH: u64 = 1048576; // 1MB

pub struct RendererSpecificAttributes {
    pub(super) maximum_entity_length: u64,
}

impl Default for RendererSpecificAttributes {
    fn default() -> Self {
        Self {
            maximum_entity_length: MAXIMUM_ENTITY_LENGTH,
        }
    }
}

pub trait WGPURendererBuilder {
    fn set_maximum_entity_length(&mut self, maximum_entity_length: u64);
}

impl WGPURendererBuilder for RendererBuilder {
    fn set_maximum_entity_length(&mut self, maximum_entity_length: u64) {
        self.renderer_specific_attributes.maximum_entity_length = maximum_entity_length;
    }
}
