use std::rc::Rc;

use examples_common::CustomEvent;
use threerender::entity::{EntityDescriptor, EntityList, EntityRendererState};
use threerender::math::Vec3;
use threerender::mesh::traits::EntityMesh;
use threerender::mesh::{PolygonMode, Sphere, Square};
#[cfg(feature = "wgpu")]
use threerender::renderer::wgpu_builder::WGPURendererBuilder;
use threerender::renderer::Updater;
use threerender::unit::{Rotation, Scale, RGBA};
use threerender::{
    CameraStyle, LightBaseStyle, LightStyle, RendererBuilder, RendererState, SceneStyle,
};
#[cfg(feature = "wgpu")]
use wgpu::Features;

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
        // Rotate light
        let prev_light_rotate_y = scene.get_light("directional").unwrap().base().rotation_y();
        scene
            .get_light_mut("directional")
            .unwrap()
            .base_mut()
            .rotate_y(prev_light_rotate_y - 0.05);

        for entity in entity_list.items_mut() {
            // Scale sphere
            if entity.id() == "sphere" {
                if entity
                    .dimension()
                    .as_glam()
                    .cmpgt(glam::Vec3::new(2., 2., 2.))
                    .all()
                {
                    self.state.should_scale_sphere = false;
                } else if entity
                    .dimension()
                    .as_glam()
                    .cmple(glam::Vec3::new(1., 1., 1.))
                    .all()
                {
                    self.state.should_scale_sphere = true;
                };
                let scale = if self.state.should_scale_sphere {
                    0.01
                } else {
                    -0.01
                };
                let prev_x = entity.scale_x();
                let prev_y = entity.scale_y();
                let prev_z = entity.scale_z();
                entity.scale_to_x(prev_x + scale);
                entity.scale_to_y(prev_y + scale);
                entity.scale_to_z(prev_z + scale);
            }

            // Rotate square
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

    renderer_builder.set_camera(CameraStyle {
        width: width as f32,
        height: height as f32,
        ..Default::default()
    });

    #[cfg(feature = "wgpu")]
    renderer_builder.set_features(Features::POLYGON_MODE_LINE);

    renderer_builder.push_state(RendererState {
        polygon_mode: PolygonMode::Line,
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
        mesh: sphere,
        fill_color: RGBA::new(255, 255, 255, 255),
        position: Vec3::ZERO,
        dimension: Vec3::ONE,
        rotation: Vec3::ZERO,
        state: EntityRendererState {
            polygon_mode: PolygonMode::Line,
            ..Default::default()
        },
        reflection: Default::default(),
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
        reflection: Default::default(),
    });
    renderer_builder.push(EntityDescriptor {
        id: "square2".to_owned(),
        mesh: square,
        fill_color: RGBA::new(255, 0, 0, 255),
        position: Vec3::new(-3., 0., -1.),
        dimension: Vec3::ONE,
        rotation: Vec3::ZERO,
        state: Default::default(),
        reflection: Default::default(),
    });

    examples_common::start(renderer_builder, Box::new(App::new()));
}
