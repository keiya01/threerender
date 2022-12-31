#[cfg(feature = "wgpu")]
mod _wgpu;
#[cfg(feature = "wgpu")]
pub use _wgpu::*;

use crate::{entity::Entity, SceneStyle};

pub trait Updater {
    fn update(&mut self, entities: &mut [Entity], scene: &mut SceneStyle);
}
