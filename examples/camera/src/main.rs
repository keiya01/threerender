use std::rc::Rc;

use examples_common::CustomEvent;
use threerender::entity::{EntityDescriptor, EntityList, ReflectionStyle};
use threerender::math::Vec3;
use threerender::mesh::traits::EntityMesh;
use threerender::mesh::{Plane, Sphere, Square};
use threerender::renderer::Updater;
use threerender::unit::{Rotation, Translation, RGB, RGBA};
use threerender::{
    CameraStyle, HemisphereLightStyle, LightBaseStyle, LightStyle, RendererBuilder, SceneStyle,
    ShadowOptions, ShadowStyle,
};

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
        match event {
            CustomEvent::MouseMove(pos) => {
                let distance_x = (pos.x / self.width * 10.) as f32;
                let distance_y = (pos.y / self.height * 10.) as f32;
                scene.camera_mut().position_mut().translate_x(distance_x);
                scene.camera_mut().position_mut().translate_y(distance_y);
            }
            CustomEvent::MouseWheel(pos) => {
                let prev = scene.camera().position().translation_z();
                scene
                    .camera_mut()
                    .position_mut()
                    .translate_z(prev + if pos.y > 0. { 0.1 } else { -0.1 });
            }
            CustomEvent::Resize(w, h) => {
                self.width = w as f64;
                self.height = h as f64;
            }
            _ => {}
        }

        for entity in entity_list.items_mut() {
            // Rotate square
            if entity.id() == "square1" {
                let prev = entity.rotation_z();
                entity.rotate_z(prev + 0.01);
            }
            if entity.id() == "square2" {
                let prev = entity.rotation_y();
                entity.rotate_y(prev + 0.01);
            }
        }
    }
}

fn main() {
    let (width, height) = (2000, 1500);
    let mut renderer_builder = RendererBuilder::new();
    renderer_builder.set_width(width);
    renderer_builder.set_height(height);
    renderer_builder.set_background(RGBA::new(137, 189, 222, 255));

    renderer_builder.set_camera(CameraStyle {
        width: width as f32,
        height: height as f32,
        far: 1000.,
        ..Default::default()
    });

    renderer_builder.set_shadow_options(ShadowOptions {
        map_size: (1028, 1028),
    });

    renderer_builder.add_light(LightStyle::with_directional(
        "directional1".to_owned(),
        LightBaseStyle {
            position: Vec3::new(5., 6., 5.),
            ..Default::default()
        },
        Some(ShadowStyle {
            far: 1000.,
            fov: 65.,
            ..Default::default()
        }),
    ));

    renderer_builder.add_light(LightStyle::with_directional(
        "directional2".to_owned(),
        LightBaseStyle {
            position: Vec3::new(-5., 6., 5.),
            ..Default::default()
        },
        Some(ShadowStyle {
            far: 1000.,
            fov: 65.,
            ..Default::default()
        }),
    ));

    renderer_builder.add_light(LightStyle::with_hemisphere(
        "hemisphere".to_owned(),
        HemisphereLightStyle {
            sky_color: RGB::new(232, 244, 252),
            ground_color: RGB::new(216, 210, 205),
        },
        Vec3::new(0., 1., 0.),
    ));

    let plane = Plane::new([0, 1, 0]);
    let plane = Rc::new(plane.use_entity());
    renderer_builder.push(EntityDescriptor {
        id: "plane".to_owned(),
        mesh: plane,
        fill_color: RGBA::new(163, 104, 64, 255),
        position: Vec3::new(-3., -2., -3.),
        dimension: Vec3::new(30., 30., 30.),
        rotation: Vec3::new(0., -1., 0.),
        state: Default::default(),
        reflection: Default::default(),
    });
    let sphere = Sphere::new(50, 50);
    let sphere = Rc::new(sphere.use_entity());
    renderer_builder.push(EntityDescriptor {
        id: "sphere".to_owned(),
        mesh: sphere,
        fill_color: RGBA::new(255, 25, 255, 255),
        position: Vec3::ZERO,
        dimension: Vec3::ONE,
        rotation: Vec3::ZERO,
        state: Default::default(),
        reflection: ReflectionStyle {
            brightness: 10.,
            shininess: 100.,
            specular: 1.,
        },
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
        reflection: ReflectionStyle {
            brightness: 10.,
            shininess: 100.,
            specular: 1.,
        },
    });
    renderer_builder.push(EntityDescriptor {
        id: "square2".to_owned(),
        mesh: square,
        fill_color: RGBA::new(255, 0, 0, 255),
        position: Vec3::new(-3., 0., -1.),
        dimension: Vec3::ONE,
        rotation: Vec3::ZERO,
        state: Default::default(),
        reflection: ReflectionStyle {
            brightness: 0.,
            shininess: 0.,
            specular: 0.1,
        },
    });

    examples_common::start(
        renderer_builder,
        Box::new(App::new(width as f64, height as f64)),
    );
}
