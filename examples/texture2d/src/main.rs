use std::rc::Rc;

use examples_common::{CustomEvent, Updater};
use threerender::color::rgb::RGBA;
use threerender::math::trs::Rotation;
use threerender::math::{Quat, Transform, Vec3};
use threerender::mesh::{BuiltInEntityOption, Plane, Sphere, Square};
use threerender::renderer::Renderer;
use threerender::traits::entity::{EntityDescriptor, EntityRendererState};
use threerender::traits::image::DefaultImage;
use threerender::{CameraStyle, LightBaseStyle, LightStyle, RendererBuilder, ShadowStyle};

struct App;

impl Updater for App {
    type Event = CustomEvent;

    fn update(&mut self, renderer: &mut Renderer, _event: Self::Event) {
        for entity in renderer.entities_mut() {
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

    let plane = Rc::new(Plane::new([0, 1, 0], None));
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

    let im = include_bytes!("../sample.jpg");

    let square = Rc::new(Square::new(Some(BuiltInEntityOption { use_texture: true })));
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
        texture: Some(Rc::new(
            DefaultImage::from_buffer(im).expect("Image load error"),
        )),
        ..Default::default()
    });

    let plane = Rc::new(Plane::new(
        [0, 1, 0],
        Some(BuiltInEntityOption { use_texture: true }),
    ));
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
        texture: Some(Rc::new(
            DefaultImage::from_buffer(im).expect("Image load error"),
        )),
        ..Default::default()
    });

    let sphere = Rc::new(Sphere::new(
        50,
        50,
        Some(BuiltInEntityOption { use_texture: true }),
    ));
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
        texture: Some(Rc::new(
            DefaultImage::from_buffer(include_bytes!("../globe.jpg")).expect("Image load error"),
        )),
        ..Default::default()
    });
    examples_common::start(renderer_builder, Box::new(App));
}
