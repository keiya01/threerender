use std::rc::Rc;

use threerender::entity::{Entity, EntityDescriptor};
use threerender::math::{Mat4, Vec3};
use threerender::mesh::{Sphere, Square};
use threerender::renderer::Updater;
use threerender::unit::{Position, RGBA};
use threerender::{RendererBuilder, SceneStyle};

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
    fn update(&mut self, entities: &mut [Entity], scene: &mut SceneStyle) {
        // Rotate light
        *(scene.light.position.inner_mut()) *= Mat4::from_rotation_y(-0.01);

        for entity in entities {
            // Scale sphere
            if entity.id == "sphere" {
                if entity.coordinates.inner().determinant() >= 2. {
                    self.state.should_scale_sphere = false;
                } else if entity.coordinates.inner().determinant() <= 1. {
                    self.state.should_scale_sphere = true;
                };
                let scale = if self.state.should_scale_sphere {
                    1.001
                } else {
                    0.999
                };
                *(entity.coordinates.inner_mut()) *=
                    Mat4::from_scale(Vec3::new(scale, scale, scale));
            }

            // Rotate square
            if entity.id == "square1" {
                *(entity.coordinates.inner_mut()) *= Mat4::from_rotation_z(0.01);
            }
            if entity.id == "square2" {
                *(entity.coordinates.inner_mut()) *= Mat4::from_rotation_y(0.01);
            }
        }
    }
}

fn main() {
    let mut renderer_builder = RendererBuilder::new();

    let sphere = Rc::new(Sphere::new(50, 50));
    renderer_builder.push(EntityDescriptor {
        id: "sphere".to_owned(),
        mesh: sphere,
        fill_color: RGBA::new(255, 255, 255, 255),
        coordinates: Position::IDENTITY,
    });
    let square = Rc::new(Square::new());
    renderer_builder.push(EntityDescriptor {
        id: "square1".to_owned(),
        mesh: square.clone(),
        fill_color: RGBA::new(0, 255, 0, 255),
        coordinates: Position::new(0., 0., -3.),
    });
    renderer_builder.push(EntityDescriptor {
        id: "square2".to_owned(),
        mesh: square,
        fill_color: RGBA::new(255, 0, 0, 255),
        coordinates: Position::new(-3., 0., -1.),
    });

    examples_common::start(renderer_builder, Box::new(App::new()));
}
