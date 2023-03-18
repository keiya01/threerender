mod camera;
mod light;

use std::f32::consts;

use getset::{Getters, MutGetters};
use glam::Mat4;

pub use camera::*;
pub use light::*;

use crate::unit::{Rotation, Translation};

#[derive(Getters, MutGetters)]
pub struct Scene {
    pub(super) lights: Vec<LightStyle>,
    #[getset(get = "pub", get_mut = "pub")]
    pub(super) camera: CameraStyle,
    #[getset(get = "pub", get_mut = "pub")]
    pub(super) shadow_options: Option<ShadowOptions>,
    #[getset(get = "pub", get_mut = "pub")]
    pub(super) max_light_num: u32,
}

impl Default for Scene {
    fn default() -> Self {
        let lights = vec![LightStyle::default()];

        Self {
            lights,
            camera: Default::default(),
            shadow_options: Default::default(),
            max_light_num: 10,
        }
    }
}

impl Scene {
    pub fn get_light(&self, id: &str) -> Option<&LightStyle> {
        self.lights.iter().find(|l| l.id() == id)
    }

    pub fn get_light_mut(&mut self, id: &str) -> Option<&mut LightStyle> {
        self.lights.iter_mut().find(|l| l.id() == id)
    }
}

#[derive(Getters, MutGetters)]
pub struct ShadowStyle {
    #[getset(get = "pub", get_mut = "pub")]
    pub fov: f32,
    #[getset(get = "pub", get_mut = "pub")]
    pub near: f32,
    #[getset(get = "pub", get_mut = "pub")]
    pub far: f32,
    #[getset(get = "pub")]
    pub center: CameraCenter,
    #[getset(get = "pub")]
    pub up: CameraUp,
}

impl Default for ShadowStyle {
    fn default() -> Self {
        Self {
            fov: 50.,
            near: 1.,
            far: 1000.,
            center: CameraCenter::default(),
            up: CameraUp::default(),
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

#[derive(Getters, MutGetters)]
pub struct ShadowOptions {
    /// Defines resolution of shadow texture
    #[getset(get = "pub", get_mut = "pub")]
    pub map_size: (u32, u32),
}

impl Default for ShadowOptions {
    fn default() -> Self {
        Self {
            map_size: (512, 512),
        }
    }
}
