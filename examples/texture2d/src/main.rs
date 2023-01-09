use std::rc::Rc;

use examples_common::CustomEvent;
use image::EncodableLayout;
use threerender::entity::{EntityDescriptor, EntityList, EntityRendererState};
use threerender::math::Vec3;
use threerender::mesh::traits::Texture2DMesh;
use threerender::mesh::{MeshType, Square, Texture2DDescriptor, Texture2DFormat};
#[cfg(feature = "wgpu")]
use threerender::renderer::builder::WGPURendererBuilder;
use threerender::renderer::Updater;
use threerender::unit::RGBA;
use threerender::{CameraStyle, RendererBuilder, RendererState, SceneStyle};
#[cfg(feature = "wgpu")]
use wgpu::Features;

struct App;

impl Updater for App {
    type Event = CustomEvent;

    fn update(
        &mut self,
        entity_list: &mut dyn EntityList,
        _scene: &mut SceneStyle,
        _event: Self::Event,
    ) {
        for entity in entity_list.items_mut() {
            // Rotate square
            if entity.id == "square" {
                entity.rotation.y += 0.01;
            }
        }
    }
}

fn main() {
    let (width, height) = (2000, 1500);
    let mut renderer_builder = RendererBuilder::new();
    renderer_builder.set_width(width);
    renderer_builder.set_height(height);

    renderer_builder.set_camera(CameraStyle {
        width: width as f32,
        height: height as f32,
        ..Default::default()
    });

    #[cfg(feature = "wgpu")]
    renderer_builder.set_features(Features::TEXTURE_BINDING_ARRAY);

    renderer_builder.push_state(RendererState {
        mesh_type: MeshType::Texture2D,
        ..Default::default()
    });

    let im = image::load_from_memory(include_bytes!("../sample.jpg")).unwrap();
    let im = im.to_rgba8();
    let (width, height) = im.dimensions();

    let square = Square::new();
    let square = Rc::new(square.use_texture2d(Texture2DDescriptor {
        width,
        height,
        format: Texture2DFormat::RGBA,
        data: im.as_bytes().to_vec(),
    }));
    renderer_builder.push(EntityDescriptor {
        id: "square".to_owned(),
        mesh: square,
        fill_color: RGBA::new(0, 255, 0, 255),
        position: Vec3::ZERO,
        dimension: Vec3::ONE,
        rotation: Vec3::ZERO,
        state: EntityRendererState {
            mesh_type: MeshType::Texture2D,
            ..Default::default()
        },
    });
    examples_common::start(renderer_builder, Box::new(App));
}
