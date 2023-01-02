use glam::{Mat4, Vec3};

pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

#[derive(Debug)]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RGBA {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Debug)]
pub struct Position(Mat4);

impl Position {
    pub const ZERO: Self = Position(Mat4::ZERO);
    pub const IDENTITY: Self = Position(Mat4::IDENTITY);
    pub const NAN: Self = Position(Mat4::NAN);

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Position(Mat4::from_translation(Vec3::new(x, y, z)))
    }

    pub fn to_array3(&self) -> [f32; 3] {
        let v = self.0.transform_vector3(Vec3::ONE);
        [v.x, v.y, v.z]
    }

    pub fn inner(&self) -> &Mat4 {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut Mat4 {
        &mut self.0
    }
}
