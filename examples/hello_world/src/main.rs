use std::rc::Rc;

use threerender::math::{self, Mat4, Vec4};
use threerender::entity::{Entity, EntityDescriptor};
use threerender::mesh::Square;
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
        let Position { x, y, z } = scene.light.position;
        let res = (Mat4::from_rotation_y(-0.01) * Vec4::new(x, y, z, 1.)).to_array();
        scene.light.position = Position::new(res[0], res[1], res[2]);

        // Rotate entity
        for entity in entities {
            *(entity.coordinates_mut()) *= Mat4::from_rotation_y(0.001);
        }
    }
}

fn main() {
    let mut renderer_builder = RendererBuilder::new();

    let square = Rc::new(Square::new());
    renderer_builder.push(EntityDescriptor {
        mesh: square.clone(),
        fill_color: RGBA::new(255, 255, 255, 255),
        coordinates: Mat4::IDENTITY,
    });
    renderer_builder.push(EntityDescriptor {
        mesh: square.clone(),
        fill_color: RGBA::new(0, 255, 0, 255),
        coordinates: Mat4::from_translation(math::Vec3::new(0., 0., -3.)),
    });
    renderer_builder.push(EntityDescriptor {
        mesh: square,
        fill_color: RGBA::new(255, 0, 0, 255),
        coordinates: Mat4::from_translation(math::Vec3::new(-3., 0., -1.)),
    });

    examples_common::start(renderer_builder, Box::new(App::new()));
}
