use std::rc::Rc;

use examples_common::CustomEvent;
use threerender::entity::{EntityDescriptor, EntityList};
use threerender::math::{Vec3, Quat};
use threerender::mesh::EntityMesh;
use threerender::mesh::{Sphere, Square};
use threerender::renderer::Updater;
use threerender::unit::{Rotation, Scale, RGBA};
use threerender::{CameraStyle, LightBaseStyle, LightStyle, RendererBuilder, Scene};

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

    fn update(&mut self, entity_list: &mut dyn EntityList, scene: &mut Scene, _event: Self::Event) {
        // Rotate light
        scene
            .get_light_mut("directional")
            .unwrap()
            .base_mut()
            .rotate_y(-0.05);

        for entity in entity_list.items_mut() {
            // Scale sphere
            if entity.id() == "sphere" {
                if entity
                    .scale()
                    .as_glam()
                    .cmpgt(glam::Vec3::new(2., 2., 2.))
                    .all()
                {
                    self.state.should_scale_sphere = false;
                } else if entity
                    .dimension()
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
            if entity.id() == "square1" {
                entity.rotate_z(0.01);
            }
            if entity.id() == "square2" {
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

    let sphere = Sphere::new(50, 50);
    let sphere = Rc::new(sphere.use_entity());
    renderer_builder.push(EntityDescriptor {
        id: "sphere".to_owned(),
        mesh: sphere,
        fill_color: RGBA::new(255, 255, 255, 255),
        position: Vec3::ZERO,
        dimension: Vec3::ONE,
        rotation: Quat::default(),
        state: Default::default(),
        reflection: Default::default(),
    });
    let square = Square::new();
    let square = Rc::new(square.use_entity());
    renderer_builder.push(EntityDescriptor {
        id: "square1".to_owned(),
        mesh: square.clone(),
        fill_color: RGBA::new(0, 255, 0, 255),
        position: Vec3::new(0., 0., -3.),
        dimension: Vec3::ONE,
        rotation: Quat::default(),
        state: Default::default(),
        reflection: Default::default(),
    });
    renderer_builder.push(EntityDescriptor {
        id: "square2".to_owned(),
        mesh: square,
        fill_color: RGBA::new(255, 0, 0, 255),
        position: Vec3::new(-3., 0., -1.),
        dimension: Vec3::ONE,
        rotation: Quat::default(),
        state: Default::default(),
        reflection: Default::default(),
    });

    examples_common::start(renderer_builder, Box::new(App::new()));
}
