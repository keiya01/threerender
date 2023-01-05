use glam::Vec3;

use crate::unit::{HeadingPitchRoll, RGB};

#[derive(Default)]
pub struct SceneStyle {
    pub light: LightStyle,
}

#[derive(Default)]
pub enum LightModel {
    #[default]
    OFF,
    DiffuseReflection,
}

pub struct LightStyle {
    // The alpha chanel is always ignored. This is to align buffer for wgsl.
    pub color: RGB,
    pub position: Vec3,
    pub heading_pitch_roll: HeadingPitchRoll,
    pub brightness: f32,
    pub model: LightModel,
}

impl Default for LightStyle {
    fn default() -> Self {
        Self {
            color: RGB::new(255, 255, 255),
            position: Vec3::new(0.0, 0.5, -1.0),
            heading_pitch_roll: HeadingPitchRoll::ZERO,
            brightness: 2.,
            model: Default::default(),
        }
    }
}
