use std::rc::Rc;

use examples_common::CustomEvent;
use threerender::entity::{EntityDescriptor, EntityList, EntityRendererState};
use threerender::math::{Mat4, Vec3};
use threerender::mesh::primitive::Primitive;
use threerender::mesh::{MeshType, PointList, PointMeshType};
use threerender::renderer::Updater;
use threerender::unit::{Position, RGBA};
use threerender::{RendererBuilder, SceneStyle, RendererState};

struct App {
}

impl App {
    fn new() -> Self {
        Self {
        }
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
                *(entity.coordinates.inner_mut()) *= Mat4::from_rotation_y(0.01);
            }
        }
    }
}

fn main() {
    let mut renderer_builder = RendererBuilder::new();

    // Create line list renderer
    renderer_builder.push_state(RendererState {
        mesh_type: MeshType::LineList,
    });

    let points = vec![
        Vec3::new(0., 0., 1.), Vec3::new(1., 0., 1.),
        Vec3::new(-1., 1., 1.), Vec3::new(1., -1., 1.),
        Vec3::new(-2., 2., 1.), Vec3::new(-2., -2., 1.),
    ];
    let lines = Rc::new(PointList::new(points, PointMeshType::Line));
    renderer_builder.push(EntityDescriptor {
        id: "lines".to_owned(),
        mesh: lines.clone(),
        fill_color: RGBA::new(255, 0, 0, 255),
        coordinates: Position::new(0., 0., 0.),
        state: EntityRendererState {
            mesh_type: lines.mesh_type(),
        },
    });

    examples_common::start(renderer_builder, Box::new(App::new()));
}
