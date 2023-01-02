use std::rc::Rc;

use threerender::entity::{Entity, EntityDescriptor};
use threerender::math::Mat4;
use threerender::mesh::{Sphere, Square};
use threerender::renderer::Updater;
use threerender::unit::{Position, RGBA};
use threerender::{RendererBuilder, SceneStyle};

struct App {}

impl App {
    fn new() -> Self {
        Self {}
    }
}

impl Updater for App {
    fn update(&mut self, entities: &mut [Entity], scene: &mut SceneStyle) {
        // Rotate light
        *(scene.light.position.inner_mut()) *= Mat4::from_rotation_y(-0.01);

        // Rotate entity
        for entity in entities {
            *(entity.coordinates.inner_mut()) *= Mat4::from_rotation_y(0.001);
        }
    }
}

fn main() {
    let mut renderer_builder = RendererBuilder::new();

    let sphere = Rc::new(Sphere::new(50, 50));
    renderer_builder.push(EntityDescriptor {
        mesh: sphere,
        fill_color: RGBA::new(255, 255, 255, 255),
        coordinates: Position::IDENTITY,
    });
    let square = Rc::new(Square::new());
    renderer_builder.push(EntityDescriptor {
        mesh: square.clone(),
        fill_color: RGBA::new(0, 255, 0, 255),
        coordinates: Position::new(0., 0., -3.),
    });
    renderer_builder.push(EntityDescriptor {
        mesh: square,
        fill_color: RGBA::new(255, 0, 0, 255),
        coordinates: Position::new(-3., 0., -1.),
    });

    examples_common::start(renderer_builder, Box::new(App::new()));
}
