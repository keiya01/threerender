use crate::unit::{Position, RGB};

#[derive(Default)]
pub struct SceneStyle {
    pub light: LightStyle,
}

pub struct LightStyle {
    // The alpha chanel is always ignored. This is to align buffer for wgsl.
    pub color: RGB,
    pub position: Position,
    pub brightness: f32,
}

#[allow(unused)]
impl LightStyle {
    pub fn new(color: RGB, position: Position, brightness: f32) -> Self {
        Self {
            color,
            position,
            brightness,
        }
    }
}

impl Default for LightStyle {
    fn default() -> Self {
        Self {
            color: RGB::new(255, 255, 255),
            position: Position::new(0.0, 0.5, -1.0),
            brightness: 2.,
        }
    }
}
