use parking_lot::Mutex;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::app::AmdGui;
use crate::backend::create_ui;
use glium::glutin;

fn create_display(event_loop: &glutin::event_loop::EventLoop<()>) -> glium::Display {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize {
            width: 800.0,
            height: 600.0,
        })
        .with_title("AMD GUI");

    let context_builder = glutin::ContextBuilder::new()
        .with_depth_buffer(0)
        .with_srgb(true)
        .with_stencil_buffer(0)
        .with_vsync(true);

    glium::Display::new(window_builder, context_builder, event_loop).unwrap()
}

pub fn run_app(amd_gui: Arc<Mutex<AmdGui>>, mut receiver: UnboundedReceiver<bool>) {
    let event_loop = glutin::event_loop::EventLoop::with_user_event();
    let display = create_display(&event_loop);

    let mut egui = egui_glium::EguiGlium::new(&display);

    let proxy = event_loop.create_proxy();
    tokio::spawn(async move {
        loop {
            if receiver.recv().await.is_some() {
                if let Err(e) = proxy.send_event(()) {
                    log::error!("{:?}", e);
                }
            }
        }
    });

    event_loop.run(move |event, _, control_flow| {
        let mut redraw = || {
            egui.begin_frame(&display);

            create_ui(amd_gui.clone(), egui.ctx());

            let (needs_repaint, shapes) = egui.end_frame(&display);

            *control_flow = if needs_repaint {
                display.gl_window().window().request_redraw();
                glutin::event_loop::ControlFlow::Poll
            } else {
                glutin::event_loop::ControlFlow::Wait
            };

            {
                use glium::Surface as _;
                let mut target = display.draw();

                let color = egui::Rgba::from_rgb(0.1, 0.3, 0.2);
                target.clear_color(color[0], color[1], color[2], color[3]);
                egui.paint(&display, &mut target, shapes);
                target.finish().unwrap();
            }
        };

        match event {
            glutin::event::Event::UserEvent(_) | glutin::event::Event::RedrawRequested(_) => {
                redraw()
            }
            glutin::event::Event::WindowEvent { event, .. } => {
                if egui.is_quit_event(&event) {
                    *control_flow = glium::glutin::event_loop::ControlFlow::Exit;
                }

                egui.on_event(&event);

                display.gl_window().window().request_redraw();
            }

            _ => (),
        }
    });
}
