use egui::panel::TopBottomSide;
use egui::{Layout, PointerButton};
use epaint::TextStyle;
use parking_lot::Mutex;
use std::sync::Arc;

use crate::app::{AmdGui, Page};
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

pub fn run_app(amd_gui: Arc<Mutex<AmdGui>>) {
    let event_loop = glutin::event_loop::EventLoop::with_user_event();
    let display = create_display(&event_loop);

    let mut egui = egui_glium::EguiGlium::new(&display);

    event_loop.run(move |event, _, control_flow| {
        let mut redraw = || {
            egui.begin_frame(&display);

            egui::containers::TopBottomPanel::new(TopBottomSide::Top, "menu").show(
                egui.ctx(),
                |ui| {
                    let mut child =
                        ui.child_ui(ui.available_rect_before_wrap(), Layout::left_to_right());
                    if child
                        .add(egui::Button::new("Config").text_style(TextStyle::Heading))
                        .clicked_by(PointerButton::Primary)
                    {
                        amd_gui.lock().page = Page::Config;
                    }
                    if child
                        .add(egui::Button::new("Monitoring").text_style(TextStyle::Heading))
                        .clicked_by(PointerButton::Primary)
                    {
                        amd_gui.lock().page = Page::Monitoring;
                    }
                    if child
                        .add(egui::Button::new("Settings").text_style(TextStyle::Heading))
                        .clicked_by(PointerButton::Primary)
                    {
                        amd_gui.lock().page = Page::Settings;
                    }
                },
            );

            egui::containers::CentralPanel::default().show(egui.ctx(), |ui| {
                let mut gui = amd_gui.lock();
                let page = gui.page;
                match page {
                    Page::Config => {
                        gui.ui(ui);
                    }
                    Page::Monitoring => {
                        gui.ui(ui);
                    }
                    Page::Settings => {
                        egui.ctx().settings_ui(ui);
                    }
                }
            });

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
            glutin::event::Event::RedrawRequested(_) => redraw(),
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
