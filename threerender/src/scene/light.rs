use crate::{
    math::{Quat, Vec3},
    ShadowStyle,
};
use getset::{Getters, MutGetters, Setters};
use threerender_color::rgb::RGB;
use threerender_math::trs::{Rotation, Translation};

#[derive(Default, Clone)]
pub enum LightModel {
    #[default]
    OFF,
    Directional,
    Hemisphere,
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
    pub rotation: Quat,
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
    fn rotation(&self) -> &Quat {
        &self.rotation
    }
    fn rotation_mut(&mut self) -> &mut Quat {
        &mut self.rotation
    }
}

impl Default for LightBaseStyle {
    fn default() -> Self {
        Self {
            color: RGB::new(255, 255, 255),
            ambient: RGB::new(0, 0, 0),
            position: Vec3::new(0., 3., 2.),
            rotation: Quat::default(),
            brightness: 1.,
        }
    }
}

#[derive(Default, Getters, MutGetters)]
pub struct LightStyle {
    #[getset(get = "pub", set = "pub")]
    id: String,
    #[getset(get = "pub", get_mut = "pub")]
    base: LightBaseStyle,
    #[getset(get = "pub", get_mut = "pub")]
    hemisphere: Option<HemisphereLightStyle>,
    #[getset(get = "pub")]
    model: LightModel,
    #[getset(get = "pub", get_mut = "pub")]
    shadow: Option<ShadowStyle>,
}

impl LightStyle {
    pub fn with_directional(id: String, base: LightBaseStyle, shadow: Option<ShadowStyle>) -> Self {
        Self {
            id,
            base,
            model: LightModel::Directional,
            shadow,
            ..Default::default()
        }
    }

    pub fn with_hemisphere(id: String, hemisphere: HemisphereLightStyle, position: Vec3) -> Self {
        Self {
            id,
            base: LightBaseStyle {
                position,
                ..Default::default()
            },
            hemisphere: Some(hemisphere),
            model: LightModel::Hemisphere,
            shadow: None,
        }
    }
}
