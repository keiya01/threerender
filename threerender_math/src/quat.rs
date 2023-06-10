use std::ops::Mul;

use crate::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub fn from_axis_angle(x: f32, y: f32, z: f32, w: f32) -> Self {
        let a = glam::Quat::from_axis_angle(glam::vec3(x, y, z), w).to_array();
        Self {
            x: a[0],
            y: a[1],
            z: a[2],
            w: a[3],
        }
    }

    pub fn from_array(a: [f32; 4]) -> Self {
        let a = glam::Quat::from_array(a).to_array();
        Self {
            x: a[0],
            y: a[1],
            z: a[2],
            w: a[3],
        }
    }

    fn mul_origin(&self, a: Self) -> Self {
        Self::from_array(self.as_glam().mul_quat(a.as_glam()).to_array())
    }

    pub fn mul_vec3_origin(&self, a: Vec3) -> Vec3 {
        Vec3::from_array(&self.as_glam().mul_vec3(a.as_glam()).to_array())
    }

    pub fn as_glam(&self) -> glam::Quat {
        glam::Quat::from_xyzw(self.x, self.y, self.z, self.w)
    }
}

impl Default for Quat {
    fn default() -> Self {
        let a = glam::Quat::default().to_array();
        Self {
            x: a[0],
            y: a[1],
            z: a[2],
            w: a[3],
        }
    }
}

impl Mul<Quat> for Quat {
    type Output = Quat;

    fn mul(self, rhs: Quat) -> Self::Output {
        self.mul_origin(rhs)
    }
}

impl Mul<Vec3> for Quat {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        self.mul_vec3_origin(rhs)
    }
}
