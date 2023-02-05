mod camera;
mod light;

use std::f32::consts;

use getset::{Getters, MutGetters};
use glam::Mat4;

pub use camera::*;
pub use light::*;

use crate::unit::{Rotation, Translation};

#[derive(Default, Getters, MutGetters)]
pub struct SceneStyle {
    #[getset(get = "pub", get_mut = "pub")]
    pub(super) light: LightStyle,
    #[getset(get = "pub", get_mut = "pub")]
    pub(super) camera: CameraStyle,
    #[getset(get = "pub", get_mut = "pub")]
    pub(super) shadow: Option<ShadowStyle>,
}

#[derive(Getters, MutGetters)]
pub struct ShadowStyle {
    #[getset(get = "pub", get_mut = "pub")]
    fov: f32,
    #[getset(get = "pub", get_mut = "pub")]
    near: f32,
    #[getset(get = "pub", get_mut = "pub")]
    far: f32,
    #[getset(get = "pub")]
    center: CameraCenter,
    #[getset(get = "pub")]
    up: CameraUp,

    /// Defines resolution of shadow texture
    #[getset(get = "pub", get_mut = "pub")]
    map_size: (u32, u32),
}

impl Default for ShadowStyle {
    fn default() -> Self {
        Self {
            fov: 50.,
            near: 1.,
            far: 100.,
            center: CameraCenter::default(),
            up: CameraUp::default(),
            map_size: (512, 512),
        }
    }
}

impl ShadowStyle {
    pub(super) const DEFAULT_MAP_SIZE: (u32, u32) = (512, 512);

    pub(super) fn transform(&self, light: &LightStyle) -> Mat4 {
        let projection =
            glam::Mat4::perspective_rh(self.fov * consts::PI / 180., 1., self.near, self.far);
        let view = glam::Mat4::look_at_rh(
            light.base().translation().as_glam(),
            self.center.0.as_glam(),
            self.up.0.as_glam(),
        );
        let view = view
            .mul_mat4(&Mat4::from_rotation_x(light.base().rotation_x()))
            .mul_mat4(&Mat4::from_rotation_y(light.base().rotation_y()))
            .mul_mat4(&Mat4::from_rotation_z(light.base().rotation_z()));
        projection * view
    }
}
