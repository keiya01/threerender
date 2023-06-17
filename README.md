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
    // Described in next section
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

You need to define a window management system by yourself because threerender is just a rendering engine for 3D.  
But you can set up window system easily by using [winit](https://github.com/rust-windowing/winit) like the following.

```rust
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
