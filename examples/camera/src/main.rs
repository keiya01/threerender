use std::rc::Rc;

use examples_common::CustomEvent;
use threerender::entity::{EntityDescriptor, EntityList};
use threerender::math::Vec3;
use threerender::mesh::traits::EntityMesh;
use threerender::mesh::{Sphere, Square};
use threerender::renderer::Updater;
use threerender::unit::RGBA;
use threerender::{CameraStyle, LightModel, LightStyle, RendererBuilder, SceneStyle};

struct App {
    width: f64,
    height: f64,
}

impl App {
    fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
}

impl Updater for App {
    type Event = CustomEvent;

    fn update(
        &mut self,
        entity_list: &mut dyn EntityList,
        scene: &mut SceneStyle,
        event: Self::Event,
    ) {
        // TODO: improve this without Mat4
        // Rotate light
        scene.light.rotation.y -= 0.05;

        match event {
            CustomEvent::MouseMove(pos) => {
                let distance_x = (pos.x / self.width * 10.) as f32;
                let distance_y = (pos.y / self.height * 10.) as f32;
                scene.camera.position.x = distance_x;
                scene.camera.position.y = distance_y;
            }
            CustomEvent::MouseWheel(pos) => {
                scene.camera.position.z += if pos.y > 0. { 0.1 } else { -0.1 };
            }
            CustomEvent::Resize(w, h) => {
                self.width = w as f64;
                self.height = h as f64;
            }
            _ => {}
        }

        for entity in entity_list.items_mut() {
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

    renderer_builder.set_camera(CameraStyle {
        width: width as f32,
        height: height as f32,
        far: 1000.,
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
    });
    renderer_builder.push(EntityDescriptor {
        id: "square2".to_owned(),
        mesh: square,
        fill_color: RGBA::new(255, 0, 0, 255),
        position: Vec3::new(-3., 0., -1.),
        dimension: Vec3::ONE,
        rotation: Vec3::ZERO,
        state: Default::default(),
    });

    examples_common::start(
        renderer_builder,
        Box::new(App::new(width as f64, height as f64)),
    );
}
