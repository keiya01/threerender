use std::rc::Rc;

use examples_common::CustomEvent;
use threerender::entity::{EntityDescriptor, EntityList};
use threerender::math::Vec3;
use threerender::mesh::traits::EntityMesh;
use threerender::mesh::{Sphere, Square};
use threerender::renderer::Updater;
use threerender::unit::RGBA;
use threerender::{CameraStyle, LightModel, LightStyle, RendererBuilder, SceneStyle};

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

    fn update(
        &mut self,
        entity_list: &mut dyn EntityList,
        scene: &mut SceneStyle,
        _event: Self::Event,
    ) {
        // TODO: improve this without Mat4
        // Rotate light
        scene.light.rotation.y -= 0.05;

        for entity in entity_list.items_mut() {
            // Scale sphere
            if entity.id == "sphere" {
                if entity.dimension.cmpgt(Vec3::new(2., 2., 2.)).all() {
                    self.state.should_scale_sphere = false;
                } else if entity.dimension.cmple(Vec3::new(1., 1., 1.)).all() {
                    self.state.should_scale_sphere = true;
                };
                let scale = if self.state.should_scale_sphere {
                    0.01
                } else {
                    -0.01
                };
                entity.dimension += Vec3::new(scale, scale, scale);
            }

            // Rotate square
            if entity.id == "square1" {
                entity.rotation.z += 0.01;
            }
            if entity.id == "square2" {
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
    renderer_builder.set_background(RGBA::new(0, 0, 0, 1));

    renderer_builder.set_camera(CameraStyle {
        width: width as f32,
        height: height as f32,
        ..Default::default()
    });

    renderer_builder.set_light(LightStyle {
        model: LightModel::Directional,
        ..Default::default()
    });

    let sphere = Sphere::new(50, 50);
    let sphere = Rc::new(sphere.use_entity());
    renderer_builder.push(EntityDescriptor {
        id: "sphere".to_owned(),
        mesh: sphere,
        fill_color: RGBA::new(255, 255, 255, 255),
        position: Vec3::ZERO,
        dimension: Vec3::ONE,
        rotation: Vec3::ZERO,
        state: Default::default(),
        has_shadow: false,
    });
    let square = Square::new();
    let square = Rc::new(square.use_entity());
    renderer_builder.push(EntityDescriptor {
        id: "square1".to_owned(),
        mesh: square.clone(),
        fill_color: RGBA::new(0, 255, 0, 255),
        position: Vec3::new(0., 0., -3.),
        dimension: Vec3::ONE,
        rotation: Vec3::ZERO,
        state: Default::default(),
        has_shadow: false,
    });
    renderer_builder.push(EntityDescriptor {
        id: "square2".to_owned(),
        mesh: square,
        fill_color: RGBA::new(255, 0, 0, 255),
        position: Vec3::new(-3., 0., -1.),
        dimension: Vec3::ONE,
        rotation: Vec3::ZERO,
        state: Default::default(),
        has_shadow: false,
    });

    examples_common::start(renderer_builder, Box::new(App::new()));
}
