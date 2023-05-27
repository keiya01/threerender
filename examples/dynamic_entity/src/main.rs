use std::rc::Rc;

use examples_common::CustomEvent;
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};
use threerender::color::rgb::RGBA;
use threerender::math::{Quat, Transform, Vec3};
use threerender::mesh::Mesh;
use threerender::mesh::Sphere;
use threerender::renderer::Updater;
use threerender::traits::entity::EntityDescriptor;
use threerender::{CameraStyle, EntityList, LightBaseStyle, LightStyle, RendererBuilder, Scene};

struct App {
    sphere: Mesh,
    rng: ThreadRng,
}

impl Updater for App {
    type Event = CustomEvent;

    fn update(&mut self, entity_list: &mut dyn EntityList, _scene: &mut Scene, event: Self::Event) {
        if let CustomEvent::MouseDown = event {
            let (x, y, z): (f32, f32, f32) = (
                self.rng.gen_range((-2.)..3.),
                self.rng.gen_range((-2.)..3.),
                self.rng.gen_range((-2.)..3.),
            );
            let (r, g, b) = ((255. / x) as u8, (255. / y) as u8, (255. / z) as u8);
            entity_list.push(EntityDescriptor {
                id: format!("sphere{}", entity_list.items().len()),
                mesh: Some(self.sphere.clone()),
                fill_color: RGBA::new(r, g, b, 255),
                transform: Transform::from_translation_rotation_scale(
                    Vec3::new(x, y, z),
                    Quat::default(),
                    Vec3::ONE,
                ),
                state: Default::default(),
                reflection: Default::default(),
                children: vec![],
            })
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
        LightBaseStyle::default(),
        None,
    ));

    let sphere = Rc::new(Sphere::new(50, 50, None));
    let sphere = Mesh::Entity(sphere);
    renderer_builder.push(EntityDescriptor {
        id: "sphere".to_owned(),
        mesh: Some(sphere.clone()),
        fill_color: RGBA::new(255, 255, 255, 255),
        transform: Transform::from_translation_rotation_scale(
            Vec3::ZERO,
            Quat::default(),
            Vec3::ONE,
        ),
        state: Default::default(),
        reflection: Default::default(),
        children: vec![],
    });

    let rng = thread_rng();

    examples_common::start(renderer_builder, Box::new(App { sphere, rng }));
}
