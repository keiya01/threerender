use std::f32::consts;

use crate::{
    math::vec::Vec3,
    unit::{Rotation, Scale},
};
use getset::{Getters, MutGetters, Setters};
use glam::{Affine3A, Mat4, Quat};

use crate::unit::Translation;

pub struct CameraPosition {
    pub(crate) translation: Vec3,
    pub(crate) rotation: Vec3,
    pub(crate) scale: Vec3,
}

impl CameraPosition {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            translation: Vec3::new(x, y, z),
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        }
    }
}
impl Default for CameraPosition {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        }
    }
}
impl Translation for CameraPosition {
    fn translation(&self) -> &Vec3 {
        &self.translation
    }
    fn translation_mut(&mut self) -> &mut Vec3 {
        &mut self.translation
    }
}

impl Rotation for CameraPosition {
    fn rotation(&self) -> &Vec3 {
        &self.rotation
    }
    fn rotation_mut(&mut self) -> &mut Vec3 {
        &mut self.rotation
    }
}

impl Scale for CameraPosition {
    fn scale(&self) -> &Vec3 {
        &self.scale
    }
    fn scale_mut(&mut self) -> &mut Vec3 {
        &mut self.scale
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
            position: CameraPosition::new(3., 4., 5.),
            center: CameraCenter(Vec3::ZERO),
            up: CameraUp(Vec3::Y),
        }
    }
}

// todo: scale for camera
impl CameraStyle {
    pub(crate) fn calc_position_vec3(&self) -> Vec3 {
        let v = Affine3A::from_scale_rotation_translation(
            self.position.scale.as_glam(),
            Quat::from_rotation_x(self.position.rotation.x)
                .mul_quat(Quat::from_rotation_y(self.position.rotation.y))
                .mul_quat(Quat::from_rotation_z(self.position.rotation.z)),
            self.position.translation.as_glam(),
        )
        .transform_vector3(self.position.translation.as_glam());
        Vec3::new(v.x, v.y, v.z)
    }

    pub(crate) fn transform(&self) -> Mat4 {
        let projection = glam::Mat4::perspective_rh(
            consts::FRAC_PI_4,
            self.width / self.height,
            self.near,
            self.far,
        );
        let view = glam::Mat4::look_at_rh(
            self.calc_position_vec3().as_glam(),
            self.center.0.as_glam(),
            self.up.0.as_glam(),
        );
        projection * view
    }
}
