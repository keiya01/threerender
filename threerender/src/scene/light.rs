use glam::Vec3;

use crate::unit::RGB;

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

pub struct LightBaseStyle {
    // The alpha chanel is always ignored. This is to align buffer for wgsl.
    pub color: RGB,
    pub ambient: RGB,
    pub position: Vec3,
    pub rotation: Vec3,
    pub brightness: f32,
}

impl Default for LightBaseStyle {
    fn default() -> Self {
        Self {
            color: RGB::new(255, 255, 255),
            ambient: RGB::new(30, 30, 30),
            position: Vec3::new(0.0, 0.5, -1.0),
            rotation: Vec3::ZERO,
            brightness: 1.,
        }
    }
}

#[derive(Default)]
pub struct LightStyle {
    pub base: LightBaseStyle,
    pub reflection: Option<ReflectionLightStyle>,
    pub hemisphere: Option<HemisphereLightStyle>,
    pub(crate) model: LightModel,
}



impl LightStyle {
    pub fn with_directional(base: LightBaseStyle) -> Self {
        Self {
            base,
            model: LightModel::Directional,
            ..Default::default()
        }
    }

    pub fn with_hemisphere(
        hemisphere: HemisphereLightStyle,
        position: Vec3,
    ) -> Self {
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
