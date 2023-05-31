use std::ops::{Add, Mul, Sub};

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

    fn add_origin(&self, a: Self) -> Self {
        Vec3::from_array(&(self.as_glam() + a.as_glam()).to_array())
    }

    fn mul_origin(&self, a: Self) -> Self {
        Vec3::from_array(&(self.as_glam() * a.as_glam()).to_array())
    }

    pub fn mul_one_origin(&self, a: f32) -> Self {
        Vec3::from_array(&(self.as_glam() * a).to_array())
    }

    pub fn sub_origin(&self, a: Self) -> Self {
        Vec3::from_array(&(self.as_glam() - a.as_glam()).to_array())
    }
}

impl Add<Vec3> for Vec3 {
    type Output = Self;

    fn add(self, rhs: Vec3) -> Self::Output {
        self.add_origin(rhs)
    }
}

impl Mul<Vec3> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: Vec3) -> Self::Output {
        self.mul_origin(rhs)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        self.mul_one_origin(rhs)
    }
}

impl Sub<Vec3> for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Vec3) -> Self::Output {
        self.sub_origin(rhs)
    }
}

impl From<Vec3> for [f32; 3] {
    fn from(val: Vec3) -> Self {
        val.as_glam().into()
    }
}
