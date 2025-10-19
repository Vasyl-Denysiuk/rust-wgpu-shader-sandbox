mod camera;
mod config;
mod object;
mod renderer;

use std::sync::{Arc, Mutex};

use eframe::egui_wgpu;

pub struct App {
    shader_conf: config::ShaderConfig,
    object: object::Object,
    camera: camera::WorldCamera,
    light: renderer::LightUniform,
    viewport_size: Option<egui::Vec2>,
}

impl App {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Option<Self> {
        let wgpu_render_state = cc.wgpu_render_state.as_ref()?;
        renderer::build_pipeline(wgpu_render_state, &None, &config::phong::Phong::new());

        Some(Self {
            camera: camera::WorldCamera::new(),
            light: renderer::LightUniform::new(),
            object: object::Object::default(),
            shader_conf: config::ShaderConfig {
                active_model: Arc::new(Mutex::new(config::phong::Phong::new())),
            },
            viewport_size: None,
        })
    }

    fn reload_shader(&self, render_state: &egui_wgpu::RenderState) {
        renderer::build_pipeline(
            render_state,
            &self.object.opened_file.as_deref(),
            &*(self.shader_conf.active_model).lock().unwrap(),
        );
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let viewport_response = egui::SidePanel::left("viewport_panel")
            .resizable(false)
            .exact_width(ctx.available_rect().width() * 0.5)
            .show(ctx, |ui| {
                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                    let size = ui.available_size();
                    if self.viewport_size.map_or(true, |current| current != size) {
                        self.camera.resize(size.x / size.y);
                        let pixels_per_point = ctx.pixels_per_point();
                        let size_in_pixels = (
                            (size.x * pixels_per_point).round() as u32,
                            (size.y * pixels_per_point).round() as u32,
                        );
                        self.viewport_size = Some(size);
                        if let Some(rs) = frame.wgpu_render_state() {
                            renderer::post_process_init(rs, size_in_pixels);
                        }
                    }
                    self.camera.resize(size.x / size.y);
                    self.custom_painting(ui);
                })
            });

        if viewport_response.response.contains_pointer() {
            if ctx.input(|i| i.key_down(egui::Key::W)) {
                self.camera.forward();
            }
            if ctx.input(|i| i.key_down(egui::Key::S)) {
                self.camera.backward();
            }
            if ctx.input(|i| i.key_down(egui::Key::D)) {
                self.camera.right();
            }
            if ctx.input(|i| i.key_down(egui::Key::A)) {
                self.camera.left();
            }
            if ctx.input(|i| i.key_down(egui::Key::Space)) {
                self.camera.up();
            }
            if ctx.input(|i| i.raw.modifiers.shift) {
                self.camera.down();
            }

            ctx.input(|i| {
                if i.pointer.primary_down() {
                    self.camera.mouse_moved(i.pointer.delta());
                }
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let current = &mut self.shader_conf.active_model.lock().unwrap().as_enum();
            egui::ComboBox::from_label("Select active shading model!")
                .selected_text(format!(
                    "{:?}",
                    current
                ))
                .show_ui(ui, |ui| {
                    ui.selectable_value(current, config::ShadingModelEnum::Phong, "Phong");
                    ui.selectable_value(current, config::ShadingModelEnum::Flat, "Flat");
                });
            if *current != self.shader_conf.active_model.lock().unwrap().as_enum() {
                self.shader_conf.active_model = match current {
                    config::ShadingModelEnum::Phong => Arc::new(Mutex::new(crate::config::phong::Phong::new())),
                    config::ShadingModelEnum::Flat => Arc::new(Mutex::new(crate::config::flat::Flat::new())),
                }
            }
            if self
                .shader_conf
                .active_model
                .lock()
                .unwrap()
                .build_widget(ui)
            {
                if let Some(rs) = frame.wgpu_render_state() {
                    self.reload_shader(rs);
                }
            }
            if self.object.build_widget(ui, ctx) {
                if let Some(rs) = frame.wgpu_render_state() {
                    object::Object::update_obj(rs, &self.object.opened_file.as_deref());
                }
            };
        });
        ctx.request_repaint();
    }
}

impl App {
    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, _response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::drag());

        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            renderer::ObjectRenderCallback {
                view_projection: renderer::CameraUniform::from_camera(&self.camera),
                light: self.light,
                params: self.shader_conf.active_model.clone(),
            },
        ));
    }
}

fn main() {
    let nativeoptions = eframe::NativeOptions::default();
    eframe::run_native(
        "egui wgpu demo",
        nativeoptions,
        Box::new(|cc| Ok(Box::new(App::new(cc).unwrap()))),
    )
    .unwrap();
}
