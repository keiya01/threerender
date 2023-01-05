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
pub struct HeadingPitchRoll {
    pub heading: f32,
    pub pitch: f32,
    pub roll: f32,
}

impl HeadingPitchRoll {
    pub const ZERO: Self = Self {
        heading: 0.,
        pitch: 0.,
        roll: 0.,
    };

    pub fn new(heading: f32, pitch: f32, roll: f32) -> Self {
        Self {
            heading,
            pitch,
            roll,
        }
    }
}
