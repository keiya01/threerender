use std::rc::Rc;

use crate::math::{Vec3, Quat};
use getset::{Getters, MutGetters, Setters};

use crate::{
    mesh::{Mesh, MeshType, PolygonMode, Topology},
    unit::{Rotation, Scale, Translation, RGBA},
    RendererState,
};

pub struct EntityDescriptor {
    pub id: String,
    pub mesh: Rc<Mesh>,
    pub fill_color: RGBA,
    pub position: Vec3,
    pub dimension: Vec3,
    pub rotation: Quat,
    pub reflection: ReflectionStyle,
    pub state: EntityRendererState,
}

#[derive(Debug, Default, Getters, MutGetters, Setters)]
pub struct Entity {
    #[getset(get = "pub")]
    pub(crate) id: String,
    #[getset(get = "pub", set = "pub")]
    pub(crate) fill_color: RGBA,
    #[getset(get = "pub", set = "pub")]
    pub(crate) position: Vec3,
    #[getset(get = "pub", set = "pub")]
    pub(crate) dimension: Vec3,
    #[getset(get = "pub", set = "pub")]
    pub(crate) rotation: Quat,
    #[getset(get = "pub", get_mut = "pub")]
    pub(crate) reflection: ReflectionStyle,
    pub(super) state: EntityRendererState,
}

impl Translation for Entity {
    fn translation(&self) -> &Vec3 {
        &self.position
    }
    fn translation_mut(&mut self) -> &mut Vec3 {
        &mut self.position
    }
}

impl Rotation for Entity {
    fn rotation(&self) -> &Quat {
        &self.rotation
    }
    fn rotation_mut(&mut self) -> &mut Quat {
        &mut self.rotation
    }
}

impl Scale for Entity {
    fn scale(&self) -> &Vec3 {
        &self.dimension
    }
    fn scale_mut(&mut self) -> &mut Vec3 {
        &mut self.dimension
    }
}

pub trait EntityList {
    fn push(&mut self, descriptor: EntityDescriptor);
    fn items(&self) -> &[Entity];
    fn items_mut(&mut self) -> &mut [Entity];
}

#[derive(Hash, Default, PartialEq, Debug)]
pub struct EntityRendererState {
    pub topology: Topology,
    pub polygon_mode: PolygonMode,
    pub mesh_type: MeshType,
}

impl EntityRendererState {
    pub fn from_renderer_state(state: RendererState) -> Self {
        Self {
            topology: state.topology,
            polygon_mode: state.polygon_mode,
            mesh_type: state.mesh_type,
        }
    }
}

impl Eq for EntityRendererState {}

#[derive(Debug, Clone, Getters, MutGetters, Setters)]
pub struct ReflectionStyle {
    pub brightness: f32,
    pub shininess: f32,
    pub specular: f32,
}

impl Default for ReflectionStyle {
    fn default() -> Self {
        Self {
            brightness: 0.,
            shininess: 0.,
            specular: 1.,
        }
    }
}
