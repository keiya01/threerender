/*!

threerender is a simple 3D rendering engine.
It will target providing feature of fundamental for 3D development.

It is similar to Three.js, but this will be more extensible and work on multiple platforms.

## Usage

```rust,no_run
use std::rc::Rc;

use threerender::color::rgb::RGBA;
use threerender::math::{Quat, Transform, Vec3};
use threerender::mesh::{Sphere, Square};
use threerender::renderer::Renderer;
use threerender::traits::entity::EntityDescriptor;
use threerender::{CameraStyle, LightBaseStyle, LightStyle, RendererBuilder};

let (width, height) = (2000, 1500);
let mut renderer_builder = RendererBuilder::new();
renderer_builder.set_width(width);
renderer_builder.set_height(height);
renderer_builder.set_background(RGBA::new(0, 0, 0, 1));

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

let sphere = Rc::new(Sphere::new(50, 50, None));
renderer_builder.push(EntityDescriptor {
    id: "sphere".to_owned(),
    mesh: Some(sphere),
    fill_color: RGBA::new(255, 255, 255, 255),
    transform: Transform::from_translation_rotation_scale(
        Vec3::ZERO,
        Quat::default(),
        Vec3::ONE,
    ),
    state: Default::default(),
    reflection: Default::default(),
    children: vec![],
    ..Default::default()
});
let square = Rc::new(Square::new(None));
renderer_builder.push(EntityDescriptor {
    id: "square1".to_owned(),
    mesh: Some(square.clone()),
    fill_color: RGBA::new(0, 255, 0, 255),
    transform: Transform::from_translation_rotation_scale(
        Vec3::new(0., 0., -3.),
        Quat::default(),
        Vec3::ONE,
    ),
    state: Default::default(),
    reflection: Default::default(),
    children: vec![],
    ..Default::default()
});
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
    reflection: Default::default(),
    children: vec![],
    ..Default::default()
});

let event_loop = winit::event_loop::EventLoop::new();
let window = winit::window::Window::new(&event_loop).unwrap();
window.set_inner_size(winit::dpi::PhysicalSize::new(
    renderer_builder.width(),
    renderer_builder.height(),
));

let mut renderer = pollster::block_on(Renderer::new(renderer_builder, Some(&window)));
event_loop.run(move |event, _target, control_flow| {
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
            renderer.render();
        }
        _ => {},
    }
});
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
