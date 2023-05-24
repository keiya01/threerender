use crate::{Quat, Vec3};

pub trait Translation {
    fn translation(&self) -> &Vec3;
    fn translation_x(&self) -> f32 {
        self.translation().x
    }
    fn translation_y(&self) -> f32 {
        self.translation().y
    }
    fn translation_z(&self) -> f32 {
        self.translation().z
    }
    fn translation_mut(&mut self) -> &mut Vec3;
    fn translate_x(&mut self, x: f32) {
        self.translation_mut().x = x;
    }
    fn translate_y(&mut self, y: f32) {
        self.translation_mut().y = y;
    }
    fn translate_z(&mut self, z: f32) {
        self.translation_mut().z = z;
    }
}

pub trait Rotation {
    fn rotation(&self) -> &Quat;
    fn rotation_x(&self) -> f32 {
        self.rotation().x
    }
    fn rotation_y(&self) -> f32 {
        self.rotation().y
    }
    fn rotation_z(&self) -> f32 {
        self.rotation().z
    }
    fn rotation_mut(&mut self) -> &mut Quat;
    fn rotate_x(&mut self, x: f32) {
        *self.rotation_mut() = self
            .rotation()
            .mul(Quat::from_array(glam::Quat::from_rotation_x(x).to_array()))
    }
    fn rotate_y(&mut self, y: f32) {
        *self.rotation_mut() = self
            .rotation()
            .mul(Quat::from_array(glam::Quat::from_rotation_y(y).to_array()))
    }
    fn rotate_z(&mut self, z: f32) {
        *self.rotation_mut() = self
            .rotation()
            .mul(Quat::from_array(glam::Quat::from_rotation_z(z).to_array()))
    }
}

pub trait Scale {
    fn scale(&self) -> &Vec3;
    fn scale_x(&self) -> f32 {
        self.scale().x
    }
    fn scale_y(&self) -> f32 {
        self.scale().y
    }
    fn scale_z(&self) -> f32 {
        self.scale().z
    }
    fn scale_mut(&mut self) -> &mut Vec3;
    fn scale_to_x(&mut self, x: f32) {
        self.scale_mut().x = x;
    }
    fn scale_to_y(&mut self, y: f32) {
        self.scale_mut().y = y;
    }
    fn scale_to_z(&mut self, z: f32) {
        self.scale_mut().z = z;
    }
}
