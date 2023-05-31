use crate::{Mat4, Quat, Vec3};

#[derive(Debug, Clone)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::default(),
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    pub fn from_cols_array_2d(matrix: [[f32; 4]; 4]) -> Self {
        let mat = Mat4::from_cols_array_2d(&matrix);
        let trs = mat.to_scale_rotation_translation();
        // Convert glam to threerender's vector type
        Self {
            scale: Vec3::from_array(&trs.0.to_array()),
            rotation: Quat::from_array(trs.1.to_array()),
            translation: Vec3::from_array(&trs.2.to_array()),
        }
    }

    pub fn from_translation_rotation_scale_array(
        translation: [f32; 3],
        rotation: [f32; 4],
        scale: [f32; 3],
    ) -> Self {
        Self {
            translation: Vec3::from_array(&translation),
            rotation: Quat::from_array(rotation),
            scale: Vec3::from_array(&scale),
        }
    }

    pub fn from_translation_rotation_scale(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    pub fn mul(&self, node: &Self) -> Self {
        Self {
            translation: self.transform_point(node.translation),
            rotation: self.rotation * node.rotation,
            scale: self.scale * node.scale,
        }
    }

    pub fn transform_point(&self, mut point: Vec3) -> Vec3 {
        point = self.scale * point;
        point = self.rotation * point;
        point = self.translation + point;
        point
    }

    pub fn as_mat4(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            self.scale.as_glam(),
            self.rotation.as_glam(),
            self.translation.as_glam(),
        )
    }
}
