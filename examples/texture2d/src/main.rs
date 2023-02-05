use std::rc::Rc;

use examples_common::CustomEvent;
use image::EncodableLayout;
use threerender::entity::{EntityDescriptor, EntityList, EntityRendererState};
use threerender::math::Vec3;
use threerender::mesh::traits::{EntityMesh, TextureMesh};
use threerender::mesh::{MeshType, Plane, Sphere, Square, TextureDescriptor, TextureFormat};
#[cfg(feature = "wgpu")]
use threerender::renderer::builder::WGPURendererBuilder;
use threerender::renderer::Updater;
use threerender::unit::{Rotation, RGBA};
use threerender::{
    CameraStyle, LightBaseStyle, LightStyle, RendererBuilder, RendererState, SceneStyle,
    ShadowStyle,
};
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
            // Rotate square
            if entity.id() == "square1" {
                let prev = entity.rotation_z();
                entity.rotate_z(prev + 0.01);
            }
            if entity.id() == "square2" {
                let prev = entity.rotation_y();
                entity.rotate_y(prev + 0.01);
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

    renderer_builder.set_light(LightStyle::with_directional(LightBaseStyle {
        position: Vec3::new(3., 2., 1.),
        ..Default::default()
    }));
    renderer_builder.set_shadow(ShadowStyle::default());

    #[cfg(feature = "wgpu")]
    renderer_builder.set_features(Features::TEXTURE_BINDING_ARRAY);

    renderer_builder.push_state(RendererState {
        mesh_type: MeshType::Texture,
        ..Default::default()
    });

    let plane = Plane::new([0, 1, 0]);
    let plane = Rc::new(plane.use_entity());
    renderer_builder.push(EntityDescriptor {
        id: "plane".to_owned(),
        mesh: plane,
        fill_color: RGBA::new(255, 255, 255, 255),
        position: Vec3::new(-3., -5., -3.),
        dimension: Vec3::new(10., 10., 10.),
        rotation: Vec3::new(0., -1., 0.),
        state: Default::default(),
    });

    let im = image::load_from_memory(include_bytes!("../sample.jpg")).unwrap();
    let im = im.to_rgba8();
    let (width, height) = im.dimensions();

    let square = Square::new();
    let square = Rc::new(square.use_texture(TextureDescriptor {
        width,
        height,
        format: TextureFormat::RGBA,
        data: im.as_bytes().to_vec(),
    }));
    renderer_builder.push(EntityDescriptor {
        id: "square".to_owned(),
        mesh: square,
        fill_color: RGBA::new(0, 255, 0, 255),
        position: Vec3::new(-1., 0., -2.),
        dimension: Vec3::ONE,
        rotation: Vec3::ZERO,
        state: EntityRendererState {
            mesh_type: MeshType::Texture,
            ..Default::default()
        },
    });

    let plane = Plane::new([0, 1, 0]);
    let plane = Rc::new(plane.use_texture(TextureDescriptor {
        width,
        height,
        format: TextureFormat::RGBA,
        data: im.as_bytes().to_vec(),
    }));
    renderer_builder.push(EntityDescriptor {
        id: "plane".to_owned(),
        mesh: plane,
        fill_color: RGBA::new(0, 255, 0, 255),
        position: Vec3::new(-1., 0., 1.),
        dimension: Vec3::ONE,
        rotation: Vec3::new(0., 0.5, 0.),
        state: EntityRendererState {
            mesh_type: MeshType::Texture,
            ..Default::default()
        },
    });

    let globe_im = image::load_from_memory(include_bytes!("../globe.jpg")).unwrap();
    let globe_im = globe_im.to_rgba8();
    let (globe_width, globe_height) = globe_im.dimensions();

    let sphere = Sphere::new(50, 50);
    let sphere = Rc::new(sphere.use_texture(TextureDescriptor {
        width: globe_width,
        height: globe_height,
        format: TextureFormat::RGBA,
        data: globe_im.as_bytes().to_vec(),
    }));
    renderer_builder.push(EntityDescriptor {
        id: "sphere".to_owned(),
        mesh: sphere,
        fill_color: RGBA::new(255, 255, 255, 255),
        position: Vec3::new(2., 0., 1.),
        dimension: Vec3::ONE,
        rotation: Vec3::new(0., 0.5, 0.),
        state: EntityRendererState {
            mesh_type: MeshType::Texture,
            ..Default::default()
        },
    });
    examples_common::start(renderer_builder, Box::new(App));
}
