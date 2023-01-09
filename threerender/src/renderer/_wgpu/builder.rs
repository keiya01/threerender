use wgpu::Features;

use crate::RendererBuilder;

pub struct RendererSpecificAttributes {
    pub(super) features: Features,
    pub(super) adapter_features: bool,
}

impl Default for RendererSpecificAttributes {
    fn default() -> Self {
        Self {
            features: Features::empty(),
            adapter_features: false,
        }
    }
}

pub trait WGPURendererBuilder {
    fn set_features(&mut self, features: Features);
    fn set_adapter_features(&mut self, enable: bool);
}

impl WGPURendererBuilder for RendererBuilder {
    fn set_features(&mut self, features: Features) {
        self.renderer_specific_attributes.features = features;
    }
    fn set_adapter_features(&mut self, enable: bool) {
        self.renderer_specific_attributes.adapter_features = enable;
    }
}
