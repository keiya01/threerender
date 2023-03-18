#[cfg(feature = "wgpu")]
mod _wgpu;

#[cfg(feature = "wgpu")]
pub use _wgpu::*;

use crate::{entity::EntityList, Scene};

pub trait Updater {
    type Event;
    fn update(&mut self, entities: &mut dyn EntityList, scene: &mut Scene, event: Self::Event);
}
