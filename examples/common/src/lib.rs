use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use threerender::{
    renderer::{Renderer, Updater},
    RendererBuilder,
};

pub enum CustomEvent {
    ReDraw,
    MouseMove,
}

type StaticUpdater = Box<dyn Updater<Event = CustomEvent>>;

fn run(
    event_loop: EventLoop<()>,
    window: Window,
    renderer_builder: RendererBuilder,
    mut updater: StaticUpdater,
) {
    let mut renderer = Renderer::new(&window, renderer_builder);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                renderer.resize(size.width, size.height);

                // For macos
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                renderer.update(&mut *updater, CustomEvent::ReDraw);

                // For macos
                window.request_redraw();

                renderer.draw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

pub fn start(mut renderer_builder: RendererBuilder, updater: StaticUpdater) {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();

    let size = window.inner_size();
    renderer_builder.set_height(size.height);
    renderer_builder.set_width(size.width);

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
        wasm_bindgen_futures::spawn_local(run(event_loop, window));
    }
}
