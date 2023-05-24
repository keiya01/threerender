use std::fs::canonicalize;
use std::rc::Rc;

use examples_common::CustomEvent;
use threerender::color::rgb::{RGB, RGBA};
use threerender::math::trs::{Rotation, Scale, Translation};
use threerender::math::{Quat, Transform, Vec3};
use threerender::mesh::EntityMesh;
use threerender::mesh::Plane;
use threerender::renderer::Updater;
use threerender::traits::entity::{EntityDescriptor, ReflectionStyle};
use threerender::{
    CameraPosition, CameraStyle, EntityList, HemisphereLightStyle, LightBaseStyle, LightStyle,
    RendererBuilder, Scene, ShadowOptions, ShadowStyle,
};
use threerender_loader::fetcher::DefaultFileSystemBasedFetcher;
use threerender_loader::gltf::GltfLoader;

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

    fn update(&mut self, _entity_list: &mut dyn EntityList, scene: &mut Scene, event: Self::Event) {
        match event {
            CustomEvent::MouseDown => self.dragging = true,
            CustomEvent::MouseUp => self.dragging = false,
            CustomEvent::MouseMove(pos) => {
                if self.dragging {
                    if self.prev_click_pos != (0., 0.) {
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
        position: CameraPosition::new(0., 50., -50.),
        ..Default::default()
    });

    // TODO: This should be able to set by each light's shadow setting
    renderer_builder.set_shadow_options(ShadowOptions {
        map_size: (2056, 2056),
    });

    renderer_builder.add_light(LightStyle::with_directional(
        "directional1".to_owned(),
        LightBaseStyle {
            position: Vec3::new(50., 60., -50.),
            ..Default::default()
        },
        Some(ShadowStyle {
            far: 1000.,
            fov: 65.,
            alpha: 0.9,
            ..Default::default()
        }),
    ));

    renderer_builder.add_light(LightStyle::with_directional(
        "directional2".to_owned(),
        LightBaseStyle {
            position: Vec3::new(-50., 60., -50.),
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

    let plane = Plane::new([0, 1, 0], None);
    let plane = Rc::new(plane.use_entity());
    renderer_builder.push(EntityDescriptor {
        id: "plane".to_owned(),
        mesh: Some(plane),
        fill_color: RGBA::new(163, 104, 64, 255),
        transform: Transform::from_translation_rotation_scale(
            Vec3::new(-3., -2., 0.),
            Quat::from_axis_angle(0., -1., 0., 1.),
            Vec3::new(30., 30., 30.),
        ),
        state: Default::default(),
        reflection: Default::default(),
        children: vec![],
    });

    let gltf_loader = GltfLoader::from_byte(
        "model",
        #[cfg(feature = "duck")]
        include_bytes!("../assets/duck/Duck.gltf"),
        #[cfg(feature = "cylinder_engine")]
        include_bytes!("../assets/cylinderEngine/2CylinderEngine.gltf"),
        #[cfg(feature = "duck")]
        DefaultFileSystemBasedFetcher::with_resolve_path(
            canonicalize("./examples/gltf/assets/duck").unwrap(),
        ),
        #[cfg(feature = "cylinder_engine")]
        DefaultFileSystemBasedFetcher::with_resolve_path(
            canonicalize("./examples/gltf/assets/cylinderEngine").unwrap(),
        ),
    )
    .unwrap();
    renderer_builder.push(EntityDescriptor {
        id: "model".to_string(),
        mesh: None,
        fill_color: RGBA::default(),
        transform: Transform {
            #[cfg(feature = "duck")]
            translation: Vec3::new(0., 1., 0.),
            #[cfg(feature = "cylinder_engine")]
            translation: Vec3::new(0., 8., 0.),
            #[cfg(feature = "duck")]
            scale: Vec3::new(10., 10., 10.),
            #[cfg(feature = "cylinder_engine")]
            scale: Vec3::new(0.05, 0.05, 0.05),
            ..Default::default()
        },
        state: Default::default(),
        reflection: ReflectionStyle::default(),
        children: gltf_loader.entities,
    });

    examples_common::start(
        renderer_builder,
        Box::new(App::new(width as f64, height as f64)),
    );
}
