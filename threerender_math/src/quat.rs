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

    pub fn mul(&self, a: Self) -> Self {
        Self::from_array(self.as_glam().mul_quat(a.as_glam()).to_array())
    }

    pub fn mul_vec3(&self, a: Vec3) -> Vec3 {
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
