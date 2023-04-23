use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use threerender::{
    renderer::{Renderer, Updater},
    RendererBuilder,
};

#[derive(Copy, Clone)]
pub enum CustomEvent {
    ReDraw,
    MouseMove(PhysicalPosition<f64>),
    MouseWheel(PhysicalPosition<f64>),
    MouseDown,
    MouseUp,
    Resize(u32, u32),
}

type StaticUpdater = Box<dyn Updater<Event = CustomEvent>>;

fn run(
    event_loop: EventLoop<()>,
    window: Window,
    renderer_builder: RendererBuilder,
    mut updater: StaticUpdater,
) {
    let mut renderer = Renderer::new(&window, renderer_builder);
    let mut cur_event = CustomEvent::ReDraw;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                renderer.resize(size.width, size.height);
                cur_event = CustomEvent::Resize(size.width, size.height);

                // For macos
                window.request_redraw();
            }
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        device_id: _,
                        state: ElementState::Pressed,
                        button: MouseButton::Left,
                        ..
                    },
                ..
            } => {
                cur_event = CustomEvent::MouseDown;
                window.request_redraw();
            }
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        device_id: _,
                        state: ElementState::Released,
                        button: MouseButton::Left,
                        ..
                    },
                ..
            } => {
                cur_event = CustomEvent::MouseUp;
                window.request_redraw();
            }
            Event::WindowEvent {
                event:
                    WindowEvent::MouseWheel {
                        delta: MouseScrollDelta::PixelDelta(pos),
                        ..
                    },
                ..
            } => {
                cur_event = CustomEvent::MouseWheel(pos);
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                cur_event = CustomEvent::MouseMove(position);
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::RedrawRequested(_) => {
                renderer.update(&mut *updater, cur_event);

                // For macos
                window.request_redraw();

                renderer.draw();
            }
            Event::RedrawEventsCleared => {
                cur_event = CustomEvent::ReDraw;
            }
            _ => {}
        }
    });
}

pub fn start(renderer_builder: RendererBuilder, updater: StaticUpdater) {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    window.set_inner_size(PhysicalSize::new(
        renderer_builder.width(),
        renderer_builder.height(),
    ));

    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        // Temporarily avoid srgb formats for the swapchain on the web
        run(event_loop, window, renderer_builder, updater);
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        use winit::platform::web::WindowExtWebSys;
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
        wasm_bindgen_futures::spawn_local(run(event_loop, window, renderer_builder, updater));
    }
}
