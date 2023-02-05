use crate::math::Vec3;
use getset::{Getters, MutGetters, Setters};

use crate::unit::{Rotation, Translation, RGB};

#[derive(Default, Clone)]
pub enum LightModel {
    #[default]
    OFF,
    Directional,
    Hemisphere,
}

#[derive(Clone)]
pub struct ReflectionLightStyle {
    pub specular: RGB,
    pub shininess: f32,
}

impl Default for ReflectionLightStyle {
    fn default() -> Self {
        Self {
            specular: RGB::new(255, 255, 255),
            shininess: 10.,
        }
    }
}

#[derive(Clone)]
pub struct HemisphereLightStyle {
    pub sky_color: RGB,
    pub ground_color: RGB,
}

impl Default for HemisphereLightStyle {
    fn default() -> Self {
        Self {
            sky_color: RGB::new(255, 255, 255),
            ground_color: RGB::new(255, 255, 255),
        }
    }
}

#[derive(Getters, MutGetters, Setters)]
pub struct LightBaseStyle {
    // The alpha chanel is always ignored. This is to align buffer for wgsl.
    #[getset(get = "pub", set = "pub")]
    pub color: RGB,
    #[getset(get = "pub", set = "pub")]
    pub ambient: RGB,
    pub position: Vec3,
    pub rotation: Vec3,
    #[getset(get = "pub", set = "pub")]
    pub brightness: f32,
}

impl Translation for LightBaseStyle {
    fn translation(&self) -> &Vec3 {
        &self.position
    }
    fn translation_mut(&mut self) -> &mut Vec3 {
        &mut self.position
    }
}

impl Rotation for LightBaseStyle {
    fn rotation(&self) -> &Vec3 {
        &self.rotation
    }
    fn rotation_mut(&mut self) -> &mut Vec3 {
        &mut self.rotation
    }
}

impl Default for LightBaseStyle {
    fn default() -> Self {
        Self {
            color: RGB::new(255, 255, 255),
            ambient: RGB::new(30, 30, 30),
            position: Vec3::new(0., 0.5, -1.0),
            rotation: Vec3::ZERO,
            brightness: 1.,
        }
    }
}

#[derive(Default, Getters, MutGetters)]
pub struct LightStyle {
    #[getset(get = "pub", get_mut = "pub")]
    base: LightBaseStyle,
    #[getset(get = "pub", get_mut = "pub")]
    reflection: Option<ReflectionLightStyle>,
    #[getset(get = "pub", get_mut = "pub")]
    hemisphere: Option<HemisphereLightStyle>,
    #[getset(get = "pub")]
    model: LightModel,
}

impl LightStyle {
    pub fn with_directional(base: LightBaseStyle) -> Self {
        Self {
            base,
            model: LightModel::Directional,
            ..Default::default()
        }
    }

    pub fn with_hemisphere(hemisphere: HemisphereLightStyle, position: Vec3) -> Self {
        Self {
            base: LightBaseStyle {
                position,
                ..Default::default()
            },
            reflection: None,
            hemisphere: Some(hemisphere),
            model: LightModel::Hemisphere,
        }
    }
}
