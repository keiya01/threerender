/*!

threerender is a simple 3D rendering engine.
It will target providing feature of fundamental for 3D development.

It is similar to Three.js, but this will be more extensible and work on multiple platforms.

## Usage

```
use std::rc::Rc;

use threerender::color::rgb::{RGBA, RGB};
use threerender::math::trs::{Rotation};
use threerender::math::{Quat, Transform, Vec3};
use threerender::mesh::{Square};
use threerender::renderer::{Updater, Renderer};
use threerender::traits::entity::EntityDescriptor;
use threerender::{CameraStyle, EntityList, LightBaseStyle, LightStyle, RendererBuilder, Scene, HemisphereLightStyle};

// You can store application state in this struct.
struct App;

// Updater is a trait provided by threerender.
// You can handle your entity in `update` function defined in this trait.
impl Updater for App {
    type Event = ();

    fn update(&mut self, entity_list: &mut dyn EntityList, _scene: &mut Scene, _event: Self::Event) {
        for entity in entity_list.items_mut() {
            // Rotate square
            if entity.id == "square" {
                entity.rotate_y(0.01);
            }
        }
    }
}

// You need to define a window management system by yourself.
fn window(renderer_builder: RendererBuilder, mut updater: Box<dyn Updater<Event = ()>>) {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    window.set_inner_size(winit::dpi::PhysicalSize::new(
        renderer_builder.width(),
        renderer_builder.height(),
    ));

    let mut renderer = Renderer::new(&window, renderer_builder);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Wait;
        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::Resized(size),
                ..
            } => {
                renderer.resize(size.width, size.height);

                // For macos
                window.request_redraw();
            }
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => *control_flow = winit::event_loop::ControlFlow::Exit,
            winit::event::Event::RedrawRequested(_) => {
                renderer.update(&mut *updater, ());

                // For macos
                window.request_redraw();

                renderer.draw();
            }
            _ => {}
        }
    });
}

fn main() {
    let (width, height) = (2000, 1500);

    // RendererBuilder is a struct to set up each entity and environment.
    let mut renderer_builder = RendererBuilder::new();
    renderer_builder.set_width(width);
    renderer_builder.set_height(height);
    renderer_builder.set_background(RGBA::new(255, 255, 255, 1));

    renderer_builder.set_camera(CameraStyle {
        width: width as f32,
        height: height as f32,
        ..Default::default()
    });

    renderer_builder.add_light(LightStyle::with_directional(
        "directional".to_owned(),
        LightBaseStyle::default(),
        None,
    ));

    renderer_builder.add_light(LightStyle::with_hemisphere(
        "hemisphere".to_owned(),
        HemisphereLightStyle {
            sky_color: RGB::new(232, 244, 252),
            ground_color: RGB::new(216, 210, 205),
        },
        Vec3::new(0., 50., 0.),
    ));

    let square = Rc::new(Square::new(None));
    renderer_builder.push(EntityDescriptor {
        id: "square".to_owned(),
        mesh: Some(square.clone()),
        fill_color: RGBA::new(0, 255, 0, 255),
        transform: Transform::from_translation_rotation_scale(
            Vec3::new(0., 0., 0.),
            Quat::default(),
            Vec3::ONE,
        ),
        state: Default::default(),
        reflection: Default::default(),
        children: vec![],
        ..Default::default()
    });

    window(renderer_builder, Box::new(App))
}
```
*/

mod entity;
pub mod math;
pub mod mesh;
pub mod renderer;
mod renderer_builder;
mod scene;
mod utils;

pub use entity::*;
pub use renderer_builder::*;
pub use scene::*;
pub use threerender_color as color;
pub use threerender_traits as traits;
