use std::rc::Rc;

use examples_common::CustomEvent;
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};
use threerender::entity::{EntityDescriptor, EntityList};
use threerender::math::vec::Vec3;
use threerender::mesh::Sphere;
use threerender::mesh::{EntityMesh, Mesh};
use threerender::renderer::Updater;
use threerender::unit::RGBA;
use threerender::{CameraStyle, LightBaseStyle, LightStyle, RendererBuilder, Scene};

struct App {
    sphere: Rc<Mesh>,
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
                mesh: self.sphere.clone(),
                fill_color: RGBA::new(r, g, b, 255),
                position: Vec3::new(x, y, z),
                dimension: Vec3::ONE,
                rotation: Vec3::ZERO,
                state: Default::default(),
                reflection: Default::default(),
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

    let sphere = Sphere::new(50, 50);
    let sphere = Rc::new(sphere.use_entity());
    renderer_builder.push(EntityDescriptor {
        id: "sphere".to_owned(),
        mesh: sphere.clone(),
        fill_color: RGBA::new(255, 255, 255, 255),
        position: Vec3::ZERO,
        dimension: Vec3::ONE,
        rotation: Vec3::ZERO,
        state: Default::default(),
        reflection: Default::default(),
    });

    let rng = thread_rng();

    examples_common::start(renderer_builder, Box::new(App { sphere, rng }));
}
