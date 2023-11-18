use std::sync::Arc;

use amdguid::parking_lot::Mutex;
use amdguid::AmdGui;
use glium::glutin;
use tokio::sync::mpsc::UnboundedReceiver;

pub fn run_app(amd_gui: Arc<Mutex<AmdGui>>, _receiver: UnboundedReceiver<bool>) {
    let event_loop = glutin::event_loop::EventLoop::with_user_event();
    let display = create_display(&event_loop);

    let mut egui_glium = egui_glium::EguiGlium::new(&display);

    let proxy = event_loop.create_proxy();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(166)).await;
            proxy.send_event(()).unwrap();
        }
    });

    event_loop.run(move |event, _, control_flow| {
        let mut redraw = || {
            let quit = false;

            let needs_repaint = egui_glium.run(&display, |egui_ctx| {
                amdguid::backend::create_ui(amd_gui.clone(), egui_ctx);
            });

            *control_flow = if quit {
                glutin::event_loop::ControlFlow::Exit
            } else if needs_repaint {
                display.gl_window().window().request_redraw();
                glutin::event_loop::ControlFlow::Poll
            } else {
                glutin::event_loop::ControlFlow::Wait
            };

            {
                use glium::Surface as _;
                let mut target = display.draw();

                let color = amdguid::egui::Rgba::from_rgb(0.1, 0.3, 0.2);
                target.clear_color(color[0], color[1], color[2], color[3]);

                egui_glium.paint(&display, &mut target);

                target.finish().unwrap();
            }
        };

        {
            match event {
                glutin::event::Event::RedrawEventsCleared if cfg!(windows) => redraw(),
                glutin::event::Event::RedrawRequested(_) if !cfg!(windows) => redraw(),
                glutin::event::Event::UserEvent(()) => redraw(),

                glutin::event::Event::WindowEvent { event, .. } => {
                    use winit::event::WindowEvent;
                    if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                    }

                    egui_glium.on_event(&event);

                    display.gl_window().window().request_redraw();
                }

                _ => (),
            }
        }
    });
}

fn create_display(event_loop: &glutin::event_loop::EventLoop<()>) -> glium::Display {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_resizable(true)
        .with_fullscreen(None)
        .with_title("AMD GUI");

    let context_builder = glutin::ContextBuilder::new()
        .with_depth_buffer(0)
        .with_srgb(true)
        .with_stencil_buffer(0)
        .with_vsync(true);

    glium::Display::new(window_builder, context_builder, event_loop).unwrap()
}
