use std::rc::Rc;

use examples_common::{CustomEvent, Updater};
use threerender::color::rgb::{RGB, RGBA};
use threerender::math::trs::{Rotation, Scale, Translation};
use threerender::math::{Quat, Transform, Vec3};
use threerender::mesh::{BuiltInEntityOption, Plane, Sphere, Square};
use threerender::renderer::Renderer;
use threerender::traits::entity::{EntityDescriptor, ReflectionStyle};
use threerender::traits::image::DefaultImage;
use threerender::{
    CameraStyle, HemisphereLightStyle, LightBaseStyle, LightStyle, RendererBuilder, ShadowOptions,
    ShadowStyle,
};

fn normalize(n: f32, v: f32) -> f32 {
    if n == 0. {
        return 0.;
    }
    if n > 0. {
        v
    } else {
        -v
    }
}

struct App {
    width: f64,
    height: f64,
    dragging: bool,
    prev_click_pos: (f64, f64),
}

impl App {
    fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            dragging: false,
            prev_click_pos: (0., 0.),
        }
    }
}

impl Updater for App {
    type Event = CustomEvent;

    fn update(&mut self, renderer: &mut Renderer, event: Self::Event) {
        match event {
            CustomEvent::MouseDown => self.dragging = true,
            CustomEvent::MouseUp => self.dragging = false,
            CustomEvent::MouseMove(pos) => {
                if self.dragging {
                    if self.prev_click_pos != (0., 0.) {
                        let scene = renderer.scene_mut();
                        let distance_x = normalize((pos.x - self.prev_click_pos.0) as f32, -0.03);
                        let distance_y = normalize((pos.y - self.prev_click_pos.1) as f32, 0.3);
                        let prev_translate_y = scene.camera().position.translation_y();
                        scene.camera_mut().position_mut().rotate_y(distance_x);
                        scene
                            .camera_mut()
                            .position_mut()
                            .translate_y(prev_translate_y + distance_y);
                    }
                    self.prev_click_pos = (pos.x, pos.y);
                }
            }
            CustomEvent::MouseWheel(pos) => {
                let scene = renderer.scene_mut();
                let prev = scene.camera().position.scale_x();
                let next = prev + if pos.y > 0. { 0.05 } else { -0.05 };
                scene.camera_mut().position.scale_to_x(next);
                scene.camera_mut().position.scale_to_y(next);
                scene.camera_mut().position.scale_to_z(next);
            }
            CustomEvent::Resize(w, h) => {
                self.width = w as f64;
                self.height = h as f64;
            }
            _ => {}
        }

        for entity in renderer.entities_mut() {
            // Rotate square
            if entity.id == "square1" {
                entity.rotate_z(0.01);
            }
            if entity.id == "square2" {
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
    renderer_builder.set_background(RGBA::new(137, 189, 222, 255));

    renderer_builder.set_camera(CameraStyle {
        width: width as f32,
        height: height as f32,
        far: 1000.,
        ..Default::default()
    });

    // TODO: This should be able to set by each light's shadow setting
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

    let plane = Rc::new(Plane::new([0, 1, 0], None));
    renderer_builder.push(EntityDescriptor {
        id: "plane".to_owned(),
        mesh: Some(plane),
        fill_color: RGBA::new(163, 104, 64, 255),
        transform: Transform::from_translation_rotation_scale(
            Vec3::new(-3., -2., -3.),
            Quat::from_axis_angle(0., -1., 0., 1.),
            Vec3::new(30., 30., 30.),
        ),
        state: Default::default(),
        reflection: Default::default(),
        children: vec![],
        ..Default::default()
    });
    let sphere = Rc::new(Sphere::new(50, 50, None));
    renderer_builder.push(EntityDescriptor {
        id: "sphere".to_owned(),
        mesh: Some(sphere),
        fill_color: RGBA::new(255, 25, 255, 255),
        transform: Transform::from_translation_rotation_scale(
            Vec3::ZERO,
            Quat::default(),
            Vec3::ONE,
        ),
        state: Default::default(),
        reflection: ReflectionStyle {
            specular: 10.,
            intensity: 100.,
        },
        children: vec![],
        ..Default::default()
    });
    let square = Rc::new(Square::new(Some(BuiltInEntityOption { use_texture: true })));
    renderer_builder.push(EntityDescriptor {
        id: "square1".to_owned(),
        mesh: Some(square),
        fill_color: RGBA::new(20, 55, 0, 255),
        transform: Transform::from_translation_rotation_scale(
            Vec3::new(0., 0., -3.),
            Quat::default(),
            Vec3::ONE,
        ),
        state: Default::default(),
        reflection: ReflectionStyle {
            specular: 10.,
            intensity: 100.,
        },
        children: vec![],
        normal_map: Some(Rc::new(
            DefaultImage::from_buffer(include_bytes!("../assets/cube-normal.png"))
                .expect("Image load error"),
        )),
        ..Default::default()
    });
    let square = Rc::new(Square::new(Some(BuiltInEntityOption { use_texture: true })));
    renderer_builder.push(EntityDescriptor {
        id: "square2".to_owned(),
        mesh: Some(square),
        fill_color: RGBA::new(255, 0, 0, 255),
        transform: Transform::from_translation_rotation_scale(
            Vec3::new(-3., 0., -1.),
            Quat::default(),
            Vec3::ONE,
        ),
        state: Default::default(),
        reflection: ReflectionStyle {
            intensity: 0.,
            specular: 0.,
        },
        children: vec![],
        normal_map: Some(Rc::new(
            DefaultImage::from_buffer(include_bytes!("../assets/cube-normal.png"))
                .expect("Image load error"),
        )),
        ..Default::default()
    });

    examples_common::start(
        renderer_builder,
        Box::new(App::new(width as f64, height as f64)),
    );
}
