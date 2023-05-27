use std::fmt::Debug;

use threerender_traits::entity::EntityDescriptor;

use super::GltfMesh;

/// Set handler to execute any process at event.
pub trait GltfHandler: Debug {
    /// Executed when the entity is created
    fn on_create(
        &self,
        _descriptor: &mut EntityDescriptor,
        _mesh: Option<&GltfMesh>,
        _row: &gltf::Node,
    ) where
        Self: Sized,
    {
    }

    /// Executed after processing the root scene.
    fn after_root(&self, _descriptors: &mut Vec<EntityDescriptor>, _row: &gltf::Scene)
    where
        Self: Sized,
    {
    }
}

#[derive(Debug, Clone)]
pub struct DefaultGltfHandler;

impl GltfHandler for DefaultGltfHandler {}
