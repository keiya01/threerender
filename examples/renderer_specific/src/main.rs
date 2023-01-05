use std::rc::Rc;

use examples_common::CustomEvent;
use threerender::entity::{EntityDescriptor, EntityList, EntityRendererState};
use threerender::math::Vec3;
use threerender::mesh::{PolygonMode, Sphere, Square};
#[cfg(feature = "wgpu")]
use threerender::renderer::wgpu_builder::WGPURendererBuilder;
use threerender::renderer::Updater;
use threerender::unit::{HeadingPitchRoll, RGBA};
use threerender::{LightModel, LightStyle, RendererBuilder, RendererState, SceneStyle};
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
        // TODO: improve this without Mat4
        // Rotate light
        scene.light.heading_pitch_roll.roll -= 0.05;

        for entity in entity_list.items_mut() {
            // Scale sphere
            if entity.id == "sphere" {
                if entity.dimension.cmpgt(Vec3::new(2., 2., 2.)).all() {
                    self.state.should_scale_sphere = false;
                } else if entity.dimension.cmple(Vec3::new(1., 1., 1.)).all() {
                    self.state.should_scale_sphere = true;
                };
                let scale = if self.state.should_scale_sphere {
                    0.01
                } else {
                    -0.01
                };
                entity.dimension += Vec3::new(scale, scale, scale);
            }

            // Rotate square
            if entity.id == "square1" {
                entity.heading_pitch_roll.pitch += 0.01;
            }
            if entity.id == "square2" {
                entity.heading_pitch_roll.roll += 0.01;
            }
        }
    }
}

fn main() {
    let mut renderer_builder = RendererBuilder::new();

    #[cfg(feature = "wgpu")]
    renderer_builder.set_features(Features::POLYGON_MODE_LINE);

    renderer_builder.push_state(RendererState {
        polygon_mode: PolygonMode::Line,
        ..Default::default()
    });

    renderer_builder.set_light(LightStyle {
        model: LightModel::DiffuseReflection,
        ..Default::default()
    });

    let sphere = Rc::new(Sphere::new(50, 50));
    renderer_builder.push(EntityDescriptor {
        id: "sphere".to_owned(),
        mesh: sphere,
        fill_color: RGBA::new(255, 255, 255, 255),
        position: Vec3::ZERO,
        dimension: Vec3::ONE,
        heading_pitch_roll: HeadingPitchRoll::ZERO,
        state: EntityRendererState {
            polygon_mode: PolygonMode::Line,
            ..Default::default()
        },
    });
    let square = Rc::new(Square::new());
    renderer_builder.push(EntityDescriptor {
        id: "square1".to_owned(),
        mesh: square.clone(),
        fill_color: RGBA::new(0, 255, 0, 255),
        position: Vec3::new(0., 0., -3.),
        dimension: Vec3::ONE,
        heading_pitch_roll: HeadingPitchRoll::ZERO,
        state: Default::default(),
    });
    renderer_builder.push(EntityDescriptor {
        id: "square2".to_owned(),
        mesh: square,
        fill_color: RGBA::new(255, 0, 0, 255),
        position: Vec3::new(-3., 0., -1.),
        dimension: Vec3::ONE,
        heading_pitch_roll: HeadingPitchRoll::ZERO,
        state: Default::default(),
    });

    examples_common::start(renderer_builder, Box::new(App::new()));
}
