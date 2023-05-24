use std::rc::Rc;

use examples_common::CustomEvent;
use image::EncodableLayout;
use threerender::color::rgb::RGBA;
use threerender::math::trs::Rotation;
use threerender::math::{Quat, Transform, Vec3};
use threerender::mesh::{EntityMesh, TextureMesh};
use threerender::mesh::{Plane, Sphere, Square, TextureDescriptor, TextureFormat};
use threerender::renderer::Updater;
use threerender::traits::entity::{EntityDescriptor, EntityRendererState};
use threerender::{
    CameraStyle, EntityList, LightBaseStyle, LightStyle, RendererBuilder, Scene, ShadowStyle,
};

struct App;

impl Updater for App {
    type Event = CustomEvent;

    fn update(
        &mut self,
        entity_list: &mut dyn EntityList,
        _scene: &mut Scene,
        _event: Self::Event,
    ) {
        for entity in entity_list.items_mut() {
            // Rotate square
            // Rotate square
            if entity.id == "square1" {
                entity.rotate_z(0.01);
            }
            if entity.id == "square2" {
                entity.rotate_y(0.01);
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

    renderer_builder.add_light(LightStyle::with_directional(
        "directional".to_owned(),
        LightBaseStyle {
            position: Vec3::new(5., 5., 3.),
            ..Default::default()
        },
        Some(ShadowStyle::default()),
    ));

    let plane = Plane::new([0, 1, 0], None);
    let plane = Rc::new(plane.use_entity());
    renderer_builder.push(EntityDescriptor {
        id: "plane".to_owned(),
        mesh: Some(plane),
        fill_color: RGBA::new(255, 255, 255, 255),
        transform: Transform::from_translation_rotation_scale(
            Vec3::new(-3., -5., -3.),
            Quat::default(),
            Vec3::new(10., 10., 10.),
        ),
        state: Default::default(),
        reflection: Default::default(),
        ..Default::default()
    });

    let im = image::load_from_memory(include_bytes!("../sample.jpg")).unwrap();
    let im = im.to_rgba8();
    let (width, height) = im.dimensions();

    let square = Square::new(Some(TextureDescriptor {
        width,
        height,
        format: TextureFormat::Rgba8,
        data: im.as_bytes().to_vec(),
    }));
    let square = Rc::new(square.use_texture());
    renderer_builder.push(EntityDescriptor {
        id: "square".to_owned(),
        mesh: Some(square),
        fill_color: RGBA::new(255, 255, 0, 255),
        transform: Transform::from_translation_rotation_scale(
            Vec3::new(-1., 0., -2.),
            Quat::default(),
            Vec3::ONE,
        ),
        state: EntityRendererState::default(),
        reflection: Default::default(),
        ..Default::default()
    });

    let plane = Plane::new([0, 1, 0], Some(TextureDescriptor {
        width,
        height,
        format: TextureFormat::Rgba8,
        data: im.as_bytes().to_vec(),
    }));
    let plane = Rc::new(plane.use_texture());
    renderer_builder.push(EntityDescriptor {
        id: "plane".to_owned(),
        mesh: Some(plane),
        fill_color: RGBA::new(0, 255, 255, 255),
        transform: Transform::from_translation_rotation_scale(
            Vec3::new(-1., 0., 1.),
            Quat::from_axis_angle(0., 0.5, 0., 1.),
            Vec3::ONE,
        ),
        state: EntityRendererState::default(),
        reflection: Default::default(),
        ..Default::default()
    });

    let globe_im = image::load_from_memory(include_bytes!("../globe.jpg")).unwrap();
    let globe_im = globe_im.to_rgba8();
    let (globe_width, globe_height) = globe_im.dimensions();

    let sphere = Sphere::new(50, 50, Some(TextureDescriptor {
        width: globe_width,
        height: globe_height,
        format: TextureFormat::Rgba8,
        data: globe_im.as_bytes().to_vec(),
    }));
    let sphere = Rc::new(sphere.use_texture());
    renderer_builder.push(EntityDescriptor {
        id: "sphere".to_owned(),
        mesh: Some(sphere),
        fill_color: RGBA::new(255, 255, 255, 255),
        transform: Transform::from_translation_rotation_scale(
            Vec3::new(2., 0., 1.),
            Quat::from_axis_angle(0., 0.5, 0., 1.),
            Vec3::ONE,
        ),
        state: EntityRendererState::default(),
        reflection: Default::default(),
        ..Default::default()
    });
    examples_common::start(renderer_builder, Box::new(App));
}
