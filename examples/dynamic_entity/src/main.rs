use std::rc::Rc;

use examples_common::{CustomEvent, Updater};
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};
use threerender::color::rgb::RGBA;
use threerender::math::{Quat, Transform, Vec3};
use threerender::mesh::Mesh;
use threerender::mesh::Sphere;
use threerender::renderer::Renderer;
use threerender::traits::entity::EntityDescriptor;
use threerender::{CameraStyle, LightBaseStyle, LightStyle, RendererBuilder};

trait Random {
    fn gen(&mut self) -> f32;
}

struct Rand {
    rng: ThreadRng,
}

impl Random for Rand {
    fn gen(&mut self) -> f32 {
        self.rng.gen_range((-2.)..3.)
    }
}

struct App {
    sphere: Rc<dyn Mesh>,
    rand: Box<dyn Random>,
}

impl Updater for App {
    type Event = CustomEvent;

    fn update(&mut self, renderer: &mut Renderer, event: Self::Event) {
        if let CustomEvent::MouseDown = event {
            let (x, y, z): (f32, f32, f32) = (self.rand.gen(), self.rand.gen(), self.rand.gen());
            let (r, g, b) = ((255. / x) as u8, (255. / y) as u8, (255. / z) as u8);
            renderer.push_entity(EntityDescriptor {
                id: format!("sphere{}", renderer.entities().len()),
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
                ..Default::default()
            })
        }
    }
}

const WIDTH: u32 = 2000;
const HEIGHT: u32 = 1500;

fn build() -> (RendererBuilder, Rc<Sphere>) {
    let (width, height) = (WIDTH, HEIGHT);
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
        ..Default::default()
    });
    (renderer_builder, sphere)
}

fn main() {
    let (renderer_builder, sphere) = build();
    let rand = Rand { rng: thread_rng() };

    examples_common::start(
        renderer_builder,
        Box::new(App {
            sphere,
            rand: Box::new(rand),
        }),
    );
}

#[test]
fn test_image() {
    struct Rand {
        cnt: f32,
    }

    impl Random for Rand {
        fn gen(&mut self) -> f32 {
            self.cnt += 1.;
            if self.cnt > 1. {
                self.cnt *= -1.;
            }
            self.cnt
        }
    }

    let (renderer_builder, sphere) = build();
    let mut app = App {
        sphere,
        rand: Box::new(Rand { cnt: 0. }),
    };
    let mut renderer =
        threerender::renderer::Renderer::new::<winit::window::Window>(renderer_builder, None);

    app.update(&mut renderer, CustomEvent::MouseDown);
    app.update(&mut renderer, CustomEvent::MouseDown);
    app.update(&mut renderer, CustomEvent::MouseDown);

    renderer.render();
    let buf = renderer.load_as_image();
    let mut file = std::fs::File::create("./test.png").unwrap();
    let img = image::RgbaImage::from_raw(WIDTH, HEIGHT, buf).unwrap();
    img.write_to(&mut file, image::ImageOutputFormat::Png)
        .unwrap();
}
