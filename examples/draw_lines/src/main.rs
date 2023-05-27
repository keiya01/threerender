use examples_common::CustomEvent;
use threerender::color::rgb::RGBA;
use threerender::math::trs::Rotation;
use threerender::math::{Quat, Transform, Vec3};
use threerender::mesh::{EntityMesh, Line, Point, Topology};

use threerender::renderer::Updater;
use threerender::traits::entity::{EntityDescriptor, EntityRendererState, RendererState};
use threerender::{CameraPosition, CameraStyle, EntityList, RendererBuilder, Scene};

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
        _scene: &mut Scene,
        _event: Self::Event,
    ) {
        for entity in entity_list.items_mut() {
            // Rotate lines
            if entity.id == "lines" {
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

    renderer_builder.set_camera(CameraStyle {
        width: width as f32,
        height: height as f32,
        position: CameraPosition::new(0., 0., 10.),
        ..Default::default()
    });

    // TODO: Create this renderer automatically
    // Create line list renderer
    renderer_builder.push_state(RendererState {
        topology: Topology::LineList,
        ..Default::default()
    });

    // TODO: Create this renderer automatically
    // Create point list renderer
    renderer_builder.push_state(RendererState {
        topology: Topology::PointList,
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

    let lines = Line::new(points);
    let lines = lines.use_entity();
    let topology = lines.topology();
    renderer_builder.push(EntityDescriptor {
        id: "lines".to_owned(),
        mesh: Some(lines),
        fill_color: RGBA::new(255, 0, 0, 255),
        transform: Transform::from_translation_rotation_scale(
            Vec3::new(0., 0., 0.),
            Quat::default(),
            Vec3::ONE,
        ),
        state: EntityRendererState {
            topology,
            ..Default::default()
        },
        reflection: Default::default(),
        children: vec![],
    });

    let mut circles = vec![];
    let specificity = 1000;
    for i in 0..((360 * 5) * specificity) {
        let i = (i as f32) / (specificity as f32);
        let radian = i.to_radians();
        let (x, y) = (i / 500. * radian.cos(), i / 500. * radian.sin());
        circles.push(Vec3::new(x, y, 1.));
    }

    let points = Point::new(circles);
    let points = points.use_entity();
    let topology = points.topology();
    renderer_builder.push(EntityDescriptor {
        id: "circle".to_owned(),
        mesh: Some(points),
        fill_color: RGBA::new(0, 0, 0, 255),
        transform: Transform::from_translation_rotation_scale(
            Vec3::new(0., 0., 0.),
            Quat::default(),
            Vec3::ONE,
        ),
        state: EntityRendererState {
            topology,
            ..Default::default()
        },
        reflection: Default::default(),
        children: vec![],
    });

    examples_common::start(renderer_builder, Box::new(App::new()));
}
