mod light;

use std::f32::consts;

use glam::{Mat4, Vec3};

pub use light::*;

#[derive(Default)]
pub struct SceneStyle {
    pub light: LightStyle,
    pub camera: CameraStyle,
    pub shadow: Option<ShadowStyle>,
}

pub struct ShadowStyle {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub center: Vec3,
    pub up: Vec3,

    /// Defines resolution of shadow texture
    pub map_size: (u32, u32),
}

impl Default for ShadowStyle {
    fn default() -> Self {
        Self {
            fov: 50.,
            near: 1.,
            far: 100.,
            center: Vec3::ZERO,
            up: Vec3::Y,
            map_size: (512, 512),
        }
    }
}

impl ShadowStyle {
    pub(super) const DEFAULT_MAP_SIZE: (u32, u32) = (512, 512);

    pub(super) fn transform(&self, light: &LightStyle) -> Mat4 {
        let projection =
            glam::Mat4::perspective_rh(self.fov * consts::PI / 180., 1., self.near, self.far);
        let view = glam::Mat4::look_at_rh(light.base.position, self.center, self.up);
        let view = view
            .mul_mat4(&Mat4::from_rotation_x(light.base.rotation.x))
            .mul_mat4(&Mat4::from_rotation_y(light.base.rotation.y))
            .mul_mat4(&Mat4::from_rotation_z(light.base.rotation.z));
        projection * view
    }
}

pub struct CameraStyle {
    pub width: f32,
    pub height: f32,
    pub near: f32,
    pub far: f32,
    pub position: Vec3,
    pub center: Vec3,
    pub up: Vec3,
}

impl Default for CameraStyle {
    fn default() -> Self {
        Self {
            width: 0.,
            height: 0.,
            near: 1.,
            far: 100.,
            position: Vec3::new(3., 4., 5.),
            center: Vec3::ZERO,
            up: Vec3::Y,
        }
    }
}

impl CameraStyle {
    pub(super) fn transform(&self) -> Mat4 {
        let projection = glam::Mat4::perspective_rh(
            consts::FRAC_PI_4,
            self.width / self.height,
            self.near,
            self.far,
        );
        let view = glam::Mat4::look_at_rh(self.position, self.center, self.up);
        projection * view
    }
}
