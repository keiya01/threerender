use std::fs::canonicalize;
use std::rc::Rc;

use examples_common::{CustomEvent, Updater};
use threerender::color::rgb::{RGB, RGBA};
use threerender::math::trs::{Rotation, Scale, Translation};
use threerender::math::{Quat, Transform, Vec3};
use threerender::mesh::Plane;
use threerender::renderer::Renderer;
use threerender::traits::entity::{EntityDescriptor, ReflectionStyle};
use threerender::{
    CameraPosition, CameraStyle, HemisphereLightStyle, LightBaseStyle, LightStyle, RendererBuilder,
    ShadowOptions, ShadowStyle, ShadowType,
};
use threerender_loader::fetcher::DefaultFileSystemBasedFetcher;
use threerender_loader::gltf::{GltfHandler, GltfLoader};

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
    }
}

#[derive(Debug)]
struct OwnGltfHandler;

impl GltfHandler for OwnGltfHandler {
    fn on_create(
        &self,
        _descriptor: &mut EntityDescriptor,
        _mesh: Option<&threerender_loader::gltf::GltfMesh>,
        _row: &gltf::Node,
    ) where
        Self: Sized,
    {
        #[cfg(feature = "avocado")]
        {
            _descriptor.receive_shadow = false;
        }
    }
}

const WIDTH: u32 = 2000;
const HEIGHT: u32 = 1500;

fn build() -> RendererBuilder {
    let (width, height) = (WIDTH, HEIGHT);
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

    renderer_builder.set_msaa_samples(8);

    // TODO: This should be able to set by each light's shadow setting
    renderer_builder.set_shadow_options(ShadowOptions {
        map_size: (4096, 4096),
    });

    renderer_builder.add_light(LightStyle::with_directional(
        "directional1".to_owned(),
        LightBaseStyle {
            position: Vec3::new(50., 80., -70.),
            ..Default::default()
        },
        Some(ShadowStyle {
            far: 1000.,
            fov: 65.,
            shadow_type: ShadowType::PCSS,
            opacity: 0.8,
            ..Default::default()
        }),
    ));

    renderer_builder.add_light(LightStyle::with_directional(
        "directional2".to_owned(),
        LightBaseStyle {
            position: Vec3::new(-50., 80., -70.),
            ..Default::default()
        },
        Some(ShadowStyle {
            far: 1000.,
            fov: 65.,
            shadow_type: ShadowType::PCSS,
            opacity: 0.8,
            ..Default::default()
        }),
    ));

    renderer_builder.add_light(LightStyle::with_hemisphere(
        "hemisphere".to_owned(),
        HemisphereLightStyle {
            sky_color: RGB::new(232, 244, 252),
            ground_color: RGB::new(216, 210, 205),
        },
        Vec3::new(0., 100., 0.),
    ));

    let plane = Rc::new(Plane::new([0, 1, 0], None));
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
        ..Default::default()
    });

    let manifest_path = env!("CARGO_MANIFEST_DIR");

    println!("{}", format!("{manifest_path}/examples/gltf/assets/duck"));

    let gltf_loader = GltfLoader::from_byte(
        "model",
        #[cfg(feature = "avocado")]
        include_bytes!("../assets/avocado/Avocado.gltf"),
        #[cfg(feature = "duck")]
        include_bytes!("../assets/duck/Duck.gltf"),
        #[cfg(feature = "cylinder_engine")]
        include_bytes!("../assets/cylinderEngine/2CylinderEngine.gltf"),
        #[cfg(feature = "avocado")]
        DefaultFileSystemBasedFetcher::with_resolve_path(
            canonicalize(format!("{manifest_path}/assets/avocado")).unwrap(),
        ),
        #[cfg(feature = "duck")]
        DefaultFileSystemBasedFetcher::with_resolve_path(
            canonicalize(format!("{manifest_path}/assets/duck")).unwrap(),
        ),
        #[cfg(feature = "cylinder_engine")]
        DefaultFileSystemBasedFetcher::with_resolve_path(
            canonicalize(format!("{manifest_path}/assets/cylinderEngine")).unwrap(),
        ),
        OwnGltfHandler,
    )
    .unwrap();
    renderer_builder.push(EntityDescriptor {
        id: "model".to_string(),
        mesh: None,
        fill_color: RGBA::default(),
        transform: Transform {
            #[cfg(feature = "avocado")]
            translation: Vec3::new(0., 1., 0.),
            #[cfg(feature = "duck")]
            translation: Vec3::new(0., 1., 0.),
            #[cfg(feature = "cylinder_engine")]
            translation: Vec3::new(0., 8., 0.),
            #[cfg(feature = "avocado")]
            scale: Vec3::new(300., 300., 300.),
            #[cfg(feature = "duck")]
            scale: Vec3::new(10., 10., 10.),
            #[cfg(feature = "cylinder_engine")]
            scale: Vec3::new(0.05, 0.05, 0.05),
            ..Default::default()
        },
        state: Default::default(),
        reflection: ReflectionStyle::default(),
        children: gltf_loader.entities,
        ..Default::default()
    });

    renderer_builder
}

fn main() {
    let renderer_builder = build();
    examples_common::start(
        renderer_builder,
        Box::new(App::new(WIDTH as f64, HEIGHT as f64)),
    );
}

#[test]
fn test_image() {
    let renderer_builder = build();
    let mut renderer = threerender::renderer::Renderer::new::<winit::window::Window>(renderer_builder, None);
    renderer.render();
    let buf = renderer.load_as_image();
    let mut file = std::fs::File::create("./test.png").unwrap();
    let img = image::RgbaImage::from_raw(WIDTH, HEIGHT, buf).unwrap();
    img.write_to(&mut file, image::ImageOutputFormat::Png).unwrap();
}
