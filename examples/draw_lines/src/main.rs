use std::rc::Rc;

use examples_common::CustomEvent;
use threerender::math::Vec3;
use threerender::mesh::mesh::EntityMesh;
use threerender::mesh::{PointList, PointTopology, Topology};
use threerender::entity::{EntityDescriptor, EntityList, EntityRendererState};
use threerender::renderer::Updater;
use threerender::unit::RGBA;
use threerender::{RendererBuilder, RendererState, SceneStyle};

struct App {}

impl App {
    fn new() -> Self {
        Self {}
    }
}

impl Updater for App {
    type Event = CustomEvent;

    fn update(
        &mut self,
        entity_list: &mut dyn EntityList,
        _scene: &mut SceneStyle,
        _event: Self::Event,
    ) {
        for entity in entity_list.items_mut() {
            // Rotate lines
            if entity.id == "lines" {
                entity.rotation.y += 0.01;
            }
        }
    }
}

fn main() {
    let mut renderer_builder = RendererBuilder::new();

    // Create line list renderer
    renderer_builder.push_state(RendererState {
        topology: Topology::LineList,
        ..Default::default()
    });

    let points = vec![
        Vec3::new(0., 0., 1.),
        Vec3::new(1., 0., 1.),
        Vec3::new(-1., 1., 1.),
        Vec3::new(1., -1., 1.),
        Vec3::new(-2., 2., 1.),
        Vec3::new(-2., -2., 1.),
    ];
    let lines = PointList::new(points, PointTopology::Line);
    let lines = Rc::new(lines.use_entity());
    renderer_builder.push(EntityDescriptor {
        id: "lines".to_owned(),
        mesh: lines.clone(),
        fill_color: RGBA::new(255, 0, 0, 255),
        position: Vec3::new(0., 0., 0.),
        dimension: Vec3::ONE,
        rotation: Vec3::ZERO,
        state: EntityRendererState {
            topology: lines.topology(),
            ..Default::default()
        },
    });

    examples_common::start(renderer_builder, Box::new(App::new()));
}
