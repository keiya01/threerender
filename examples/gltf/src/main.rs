use std::fs::canonicalize;
use std::rc::Rc;

use examples_common::CustomEvent;
use threerender::entity::{EntityDescriptor, EntityList, ReflectionStyle};
use threerender::math::{Vec3, Quat};
use threerender::mesh::EntityMesh;
use threerender::mesh::Plane;
use threerender::renderer::Updater;
use threerender::unit::{Rotation, Scale, Translation, RGB, RGBA};
use threerender::{
    CameraStyle, HemisphereLightStyle, LightBaseStyle, LightStyle, RendererBuilder, Scene,
    ShadowOptions, ShadowStyle, CameraPosition,
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
                        let distance_y = normalize((pos.y - self.prev_click_pos.1) as f32, 10.);
                        let prev_translate_y = scene.camera().position.translation_y();
                        scene
                            .camera_mut()
                            .position_mut()
                            .rotate_y(distance_x);
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
        far: 10000.,
        position: CameraPosition::new(0., 800., -1000.),
        ..Default::default()
    });

    // TODO: This should be able to set by each light's shadow setting
    renderer_builder.set_shadow_options(ShadowOptions {
        map_size: (1028, 1028),
    });

    renderer_builder.add_light(LightStyle::with_directional(
        "directional1".to_owned(),
        LightBaseStyle {
            position: Vec3::new(5., 30., 20.),
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
            position: Vec3::new(-5., 30., 20.),
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
        rotation: Quat::from_axis_angle(0., -1., 0., 1.),
        state: Default::default(),
        reflection: Default::default(),
    });

    let gltf_loader = GltfLoader::from_byte(
        include_bytes!("../assets/cylinderEngine/2CylinderEngine.gltf"),
        DefaultFileSystemBasedFetcher::with_resolve_path(
            canonicalize("./examples/gltf/assets/cylinderEngine").unwrap(),
        ),
    )
    .unwrap();
    let mut idx = 1;
    for entity in gltf_loader.entities {
        let color = entity.material.map_or_else(
            || RGBA::new(255, 255, 255, 255),
            |m| {
                RGBA::from_f32(
                    m.base_color[0],
                    m.base_color[1],
                    m.base_color[2],
                    m.base_color[3],
                )
            },
        );
        let trs = &entity.transform.clone();
        let model = Rc::new(entity.use_entity());
        renderer_builder.push(EntityDescriptor {
            id: format!("model{}", idx),
            mesh: model.clone(),
            fill_color: color,
            position: trs.translation,
            dimension: trs.scale,
            rotation: trs.rotation,
            state: Default::default(),
            reflection: ReflectionStyle {
                brightness: 10.,
                shininess: 100.,
                specular: 1.,
            },
        });
        idx += 1;
    }

    examples_common::start(
        renderer_builder,
        Box::new(App::new(width as f64, height as f64)),
    );
}
