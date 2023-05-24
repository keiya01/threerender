// On this project, we are using `glam` math library,
// but we don't want to force math library.
// So we are using this struct for proxy.
#[derive(Default, Debug, Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
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

    pub fn from_array(a: &[f32; 3]) -> Self {
        Self {
            x: a[0],
            y: a[1],
            z: a[2],
        }
    }

    pub fn as_glam(&self) -> glam::Vec3 {
        glam::vec3(self.x, self.y, self.z)
    }

    pub fn add(&self, a: Self) -> Self {
        Vec3::from_array(&(self.as_glam() + a.as_glam()).to_array())
    }
    pub fn mul(&self, a: Self) -> Self {
        Vec3::from_array(&(self.as_glam() * a.as_glam()).to_array())
    }
}
