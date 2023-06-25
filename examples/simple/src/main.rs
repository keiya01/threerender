use std::rc::Rc;

use threerender::color::rgb::RGBA;
use threerender::math::{Quat, Transform, Vec3};
use threerender::mesh::{Sphere, Square};
use threerender::renderer::Renderer;
use threerender::traits::entity::EntityDescriptor;
use threerender::{CameraStyle, LightBaseStyle, LightStyle, RendererBuilder};

const WIDTH: u32 = 2000;
const HEIGHT: u32 = 1500;

async fn render() -> (
    Renderer,
    Option<winit::window::Window>,
    Option<winit::event_loop::EventLoop<()>>,
) {
    let (width, height) = (WIDTH, HEIGHT);
    let mut renderer_builder = RendererBuilder::new();
    renderer_builder.set_width(width);
    renderer_builder.set_height(height);
    renderer_builder.set_background(RGBA::new(0, 0, 0, 255));

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

    let (window, event_loop) = if cfg!(test) {
        (None, None)
    } else {
        let event_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::Window::new(&event_loop).unwrap();
        window.set_inner_size(winit::dpi::PhysicalSize::new(
            renderer_builder.width(),
            renderer_builder.height(),
        ));
        (Some(window), Some(event_loop))
    };

    let renderer = Renderer::new(renderer_builder, window.as_ref()).await;

    (renderer, window, event_loop)
}

async fn run() {
    let (mut renderer, window, event_loop) = render().await;

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.as_ref().unwrap().canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
    }

    {
        event_loop
            .unwrap()
            .run(move |event, _target, control_flow| {
                match event {
                    winit::event::Event::WindowEvent {
                        event: winit::event::WindowEvent::Resized(size),
                        ..
                    } => {
                        renderer.resize(size.width, size.height);
                        // For macos
                        window.as_ref().unwrap().request_redraw();
                    }
                    winit::event::Event::WindowEvent {
                        event: winit::event::WindowEvent::CloseRequested,
                        ..
                    } => *control_flow = winit::event_loop::ControlFlow::Exit,
                    winit::event::Event::RedrawRequested(_) => {
                        renderer.render();
                    }
                    _ => {}
                }
            });
    }
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    pollster::block_on(run());
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(run());
}

#[test]
fn test_image() {
    let (mut renderer, _, _) = pollster::block_on(render());
    renderer.render();
    let buf = renderer.load_as_image();
    let mut file = std::fs::File::create("./test.png").unwrap();
    let img = image::RgbaImage::from_raw(WIDTH, HEIGHT, buf).unwrap();
    img.write_to(&mut file, image::ImageOutputFormat::Png)
        .unwrap();
}
