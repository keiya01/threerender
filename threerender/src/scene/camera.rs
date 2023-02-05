use std::f32::consts;

use crate::math::Vec3;
use getset::{Getters, MutGetters, Setters};
use glam::Mat4;

use crate::unit::Translation;

pub struct CameraPosition(pub(crate) Vec3);
impl CameraPosition {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(Vec3::new(x, y, z))
    }
}
impl Default for CameraPosition {
    fn default() -> Self {
        Self(Vec3::ZERO)
    }
}
impl Translation for CameraPosition {
    fn translation(&self) -> &Vec3 {
        &self.0
    }
    fn translation_mut(&mut self) -> &mut Vec3 {
        &mut self.0
    }
}

pub struct CameraCenter(pub(crate) Vec3);
impl CameraCenter {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(Vec3::new(x, y, z))
    }
}
impl Default for CameraCenter {
    fn default() -> Self {
        Self(Vec3::ZERO)
    }
}
impl Translation for CameraCenter {
    fn translation(&self) -> &Vec3 {
        &self.0
    }
    fn translation_mut(&mut self) -> &mut Vec3 {
        &mut self.0
    }
}

pub struct CameraUp(pub(crate) Vec3);
impl CameraUp {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(Vec3::new(x, y, z))
    }
}
impl Default for CameraUp {
    fn default() -> Self {
        Self(Vec3::Y)
    }
}
impl Translation for CameraUp {
    fn translation(&self) -> &Vec3 {
        &self.0
    }
    fn translation_mut(&mut self) -> &mut Vec3 {
        &mut self.0
    }
}

#[derive(Getters, MutGetters, Setters)]
pub struct CameraStyle {
    #[getset(get = "pub", set = "pub")]
    pub width: f32,
    #[getset(get = "pub", set = "pub")]
    pub height: f32,
    #[getset(get = "pub", set = "pub")]
    pub near: f32,
    #[getset(get = "pub", set = "pub")]
    pub far: f32,
    #[getset(get = "pub", get_mut = "pub")]
    pub position: CameraPosition,
    #[getset(get = "pub", get_mut = "pub")]
    pub center: CameraCenter,
    #[getset(get = "pub", get_mut = "pub")]
    pub up: CameraUp,
}

impl Default for CameraStyle {
    fn default() -> Self {
        Self {
            width: 0.,
            height: 0.,
            near: 1.,
            far: 100.,
            position: CameraPosition(Vec3::new(3., 4., 5.)),
            center: CameraCenter(Vec3::ZERO),
            up: CameraUp(Vec3::Y),
        }
    }
}

impl CameraStyle {
    pub(crate) fn transform(&self) -> Mat4 {
        let projection = glam::Mat4::perspective_rh(
            consts::FRAC_PI_4,
            self.width / self.height,
            self.near,
            self.far,
        );
        let view = glam::Mat4::look_at_rh(
            self.position.0.as_glam(),
            self.center.0.as_glam(),
            self.up.0.as_glam(),
        );
        projection * view
    }
}
