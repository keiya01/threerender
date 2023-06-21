use std::rc::Rc;

use examples_common::{CustomEvent, Updater};
use threerender::color::rgb::RGBA;
use threerender::math::trs::{Rotation, Scale};
use threerender::math::{Quat, Transform, Vec3};
use threerender::mesh::{Sphere, Square};
use threerender::renderer::Renderer;
use threerender::traits::entity::EntityDescriptor;
use threerender::{CameraStyle, LightBaseStyle, LightStyle, RendererBuilder};

#[derive(Default)]
struct State {
    should_scale_sphere: bool,
}

struct App {
    state: State,
}

impl App {
    fn new() -> Self {
        Self {
            state: State {
                should_scale_sphere: true,
            },
        }
    }
}

impl Updater for App {
    type Event = CustomEvent;

    fn update(&mut self, renderer: &mut Renderer, _event: Self::Event) {
        {
            // Rotate light
            renderer
                .scene_mut()
                .get_light_mut("directional")
                .unwrap()
                .base_mut()
                .rotate_y(-0.05);
        }

        for entity in renderer.entities_mut() {
            // Scale sphere
            if entity.id == "sphere" {
                if entity
                    .scale()
                    .as_glam()
                    .cmpgt(glam::Vec3::new(2., 2., 2.))
                    .all()
                {
                    self.state.should_scale_sphere = false;
                } else if entity
                    .scale()
                    .as_glam()
                    .cmple(glam::Vec3::new(1., 1., 1.))
                    .all()
                {
                    self.state.should_scale_sphere = true;
                };
                let scale = if self.state.should_scale_sphere {
                    0.01
                } else {
                    -0.01
                };
                let prev_x = entity.scale_x();
                let prev_y = entity.scale_y();
                let prev_z = entity.scale_z();
                entity.scale_to_x(prev_x + scale);
                entity.scale_to_y(prev_y + scale);
                entity.scale_to_z(prev_z + scale);
            }

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

const WIDTH: u32 = 2000;
const HEIGHT: u32 = 1500;

fn build() -> RendererBuilder {
    let (width, height) = (WIDTH, HEIGHT);
    let mut renderer_builder = RendererBuilder::new();
    renderer_builder.set_width(width);
    renderer_builder.set_height(height);
    renderer_builder.set_background(RGBA::new(0, 0, 0, 1));

    renderer_builder.set_camera(CameraStyle {
        width: width as f32,
        height: height as f32,
        ..Default::default()
    });

    renderer_builder.add_light(LightStyle::with_directional(
        "directional".to_owned(),
        LightBaseStyle::default(),
        None,
    ));

    let sphere = Rc::new(Sphere::new(50, 50, None));
    renderer_builder.push(EntityDescriptor {
        id: "sphere".to_owned(),
        mesh: Some(sphere),
        fill_color: RGBA::new(255, 255, 255, 255),
        transform: Transform::from_translation_rotation_scale(
            Vec3::ZERO,
            Quat::default(),
            Vec3::ONE,
        ),
        state: Default::default(),
        reflection: Default::default(),
        children: vec![],
        ..Default::default()
    });
    let square = Rc::new(Square::new(None));
    renderer_builder.push(EntityDescriptor {
        id: "square1".to_owned(),
        mesh: Some(square.clone()),
        fill_color: RGBA::new(0, 255, 0, 255),
        transform: Transform::from_translation_rotation_scale(
            Vec3::new(0., 0., -3.),
            Quat::default(),
            Vec3::ONE,
        ),
        state: Default::default(),
        reflection: Default::default(),
        children: vec![],
        ..Default::default()
    });
    renderer_builder.push(EntityDescriptor {
        id: "square2".to_owned(),
        mesh: Some(square),
        fill_color: RGBA::new(255, 0, 0, 255),
        transform: Transform::from_translation_rotation_scale(
            Vec3::new(-3., 0., -1.),
            Quat::default(),
            Vec3::ONE,
        ),
        state: Default::default(),
        reflection: Default::default(),
        children: vec![],
        ..Default::default()
    });
    renderer_builder
}

fn main() {
    let renderer_builder = build();

    examples_common::start(renderer_builder, Box::new(App::new()));
}

#[test]
fn test_image() {
    let renderer_builder = build();
    let mut renderer =
        threerender::renderer::Renderer::new::<winit::window::Window>(renderer_builder, None);
    renderer.render();
    let buf = renderer.load_as_image();
    let mut file = std::fs::File::create("./test.png").unwrap();
    let img = image::RgbaImage::from_raw(WIDTH, HEIGHT, buf).unwrap();
    img.write_to(&mut file, image::ImageOutputFormat::Png)
        .unwrap();
}
