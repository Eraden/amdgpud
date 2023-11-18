use std::sync::Arc;

use amdguid::parking_lot::Mutex;
use amdguid::AmdGui;
use tokio::sync::mpsc::UnboundedReceiver;

pub fn run_app(amd_gui: Arc<Mutex<AmdGui>>, _receiver: UnboundedReceiver<bool>) {
    let clear_color = [0.1, 0.1, 0.1];

    let event_loop = glutin::event_loop::EventLoop::with_user_event();
    let (gl_window, gl) = create_display(&event_loop);
    let gl = std::rc::Rc::new(gl);

    let mut egui_glow = ::egui_glow::EguiGlow::new(gl_window.window(), gl.clone());

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

            let needs_repaint = egui_glow.run(gl_window.window(), |egui_ctx| {
                amdguid::backend::create_ui(amd_gui.clone(), egui_ctx);
            });

            *control_flow = if quit {
                glutin::event_loop::ControlFlow::Exit
            } else if needs_repaint {
                gl_window.window().request_redraw();
                glutin::event_loop::ControlFlow::Poll
            } else {
                glutin::event_loop::ControlFlow::Wait
            };

            {
                unsafe {
                    use glow::HasContext as _;
                    gl.clear_color(clear_color[0], clear_color[1], clear_color[2], 1.0);
                    gl.clear(glow::COLOR_BUFFER_BIT);
                }

                egui_glow.paint(gl_window.window());
                gl_window.swap_buffers().unwrap();
            }
        };

        match event {
            glutin::event::Event::RedrawEventsCleared if cfg!(windows) => redraw(),
            glutin::event::Event::RedrawRequested(_) if !cfg!(windows) => redraw(),
            glutin::event::Event::UserEvent(()) => redraw(),

            glutin::event::Event::WindowEvent { event, .. } => {
                use glutin::event::WindowEvent;
                if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                }

                if let WindowEvent::Resized(physical_size) = &event {
                    gl_window.resize(*physical_size);
                } else if let WindowEvent::ScaleFactorChanged { new_inner_size, .. } = &event {
                    gl_window.resize(**new_inner_size);
                }

                egui_glow.on_event(&event);

                gl_window.window().request_redraw();
            }
            glutin::event::Event::LoopDestroyed => {
                egui_glow.destroy();
            }

            _ => (),
        }
    });
}

fn create_display(
    event_loop: &glutin::event_loop::EventLoop<()>,
) -> (
    glutin::WindowedContext<glutin::PossiblyCurrent>,
    glow::Context,
) {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_resizable(true)
        .with_fullscreen(None)
        .with_title("AMD GUI");

    let gl_window = unsafe {
        glutin::ContextBuilder::new()
            .with_depth_buffer(0)
            .with_srgb(true)
            .with_stencil_buffer(0)
            .with_vsync(true)
            .build_windowed(window_builder, event_loop)
            .unwrap()
            .make_current()
            .unwrap()
    };

    let gl = unsafe { glow::Context::from_loader_function(|s| gl_window.get_proc_address(s)) };

    (gl_window, gl)
}
