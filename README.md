# threerender

**CAUTION: Currently, this is POC, and in development, so not production ready.**
**If you interest this project, you can see [examples dir](/examples)**

## Overview

threerender is a simple 3D rendering engine.
It will target providing feature of fundamental for 3D development.

It is similar to Three.js, but this will be more extensible and work on multiple platforms. 

## Usage

You can use this library as follow.

```rust
use std::rc::Rc;

use threerender::color::rgb::RGBA;
use threerender::math::{Quat, Transform, Vec3};
use threerender::mesh::{Sphere, Square};
use threerender::renderer::Renderer;
use threerender::traits::entity::EntityDescriptor;
use threerender::{CameraStyle, LightBaseStyle, LightStyle, RendererBuilder, HemisphereLightStyle};

fn main() {
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
    renderer_builder.add_light(LightStyle::with_hemisphere(
        "hemisphere".to_owned(),
        HemisphereLightStyle::default(),
        Vec3::new(0., 50., 0.),
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

    let mut renderer = Renderer::new(&window, renderer_builder);
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
                renderer.draw();
            }
            _ => {},
        }
    });
}
```

## Examples

You can try this project by [examples](/examples).

You can run these examples by below command.

```sh
cargo run -p examples_{PROJECT_NAME}
```

## Road map

- [x] 3D entities
  - [x] Square
    - [x] 2D texture rendering
  - [x] Sphere
    - [x] 2D texture rendering
- [x] 2D entities
  - [x] Lines
  - [x] Plane
    - [x] 2D texture rendering
  - [x] Triangle
- [x] Camera
- [ ] Light
  - [x] Directional light
  - [ ] Spot light
  - [ ] Point light
  - [x] Hemisphere light
- [ ] Shadow
  - [x] Directional shadow
  - [x] Opacity
  - [x] Soft shadow(PCSS)
  - [ ] Point light shadow
  - [ ] Spot light shadow
- [x] Multi light/shadow
- [x] Reflection rate for entity
- [x] 2D texture
- [x] Override shader
- [ ] glTF support
  - [x] Basic glTF support
  - [ ] Animation
  - [ ] PBR
- [x] Normal mapping
- [x] Model transparency
- [ ] Extensible window system
- [ ] Extendable shader by user
- [ ] Custom render process by implementing trait
  - [ ] Shadow rendering
  - [ ] Ray tracing
- [ ] Performance improvement
    - [ ] Multiple render target
      - https://threejs.org/docs/#api/en/renderers/WebGLMultipleRenderTargets
      - User can customize own renderer like the deferred rendering
        - https://github.com/mrdoob/three.js/issues/5180
        - https://github.com/mrdoob/three.js/issues/2624
    - [ ] dirty check
    - [ ] Optimize multi object like cloning object and transfer vertex more efficiently
    - [ ] Mip map for texture
      - Provide some functionality to be able to define mip map.
    - [ ] multi threading
    - [ ] Optimize image loading. Eg. We should not draw image if image is not visible.
    - [ ] Level of details for polygon(LOD)
- [ ] Font
- [ ] Integration with 2D library like egui
- [ ] Web support / Deno support(native binding)
- [ ] Convenient Math API for 3D development
