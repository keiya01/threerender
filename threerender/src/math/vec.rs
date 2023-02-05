// On this project, we are using `glam` math library,
// but we don't want to force math library.
// So we are using this struct for proxy.
#[derive(Default, Debug, Clone, Copy)]
pub struct Vec3 {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Vec3 {
        x: 0.,
        y: 0.,
        z: 0.,
    };
    pub const ONE: Self = Vec3 {
        x: 1.,
        y: 1.,
        z: 1.,
    };
    pub const X: Self = Vec3 {
        x: 1.,
        y: 0.,
        z: 0.,
    };
    pub const Y: Self = Vec3 {
        x: 0.,
        y: 1.,
        z: 0.,
    };
    pub const Z: Self = Vec3 {
        x: 0.,
        y: 0.,
        z: 1.,
    };

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn as_glam(&self) -> glam::Vec3 {
        glam::vec3(self.x, self.y, self.z)
    }
}
