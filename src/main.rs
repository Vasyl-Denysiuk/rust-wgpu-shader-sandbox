pub mod camera;
pub mod config;
pub mod object;
pub mod renderer;

use eframe::egui_wgpu;

pub struct App {
    shader_conf: config::ShaderConfig,
    object: object::Object,
    camera: camera::WorldCamera,
    light: renderer::LightUniform,
    model: renderer::ModelUniform,
}

impl App {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Option<Self> {
        let wgpu_render_state = cc.wgpu_render_state.as_ref()?;
        renderer::build_pipeline(wgpu_render_state, &None, &config::Phong::new());

        Some(Self {
            camera: camera::WorldCamera::new(),
            light: renderer::LightUniform::new(),
            model: renderer::ModelUniform::new(),
            object: object::Object::default(),
            shader_conf: config::ShaderConfig {
                active_model: Box::new(config::Phong::new()),
            },
        })
    }

    fn reload_shader(&self, render_state: &egui_wgpu::RenderState) {
        renderer::build_pipeline(
            render_state,
            &self.object.opened_file.as_ref().map(|p| p.as_path()),
            &*self.shader_conf.active_model,
        );
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.camera
            .resize(0.5 * ctx.available_rect().width() / ctx.available_rect().height());

        let viewport_response = egui::SidePanel::left("viewport_panel")
            .exact_width(ctx.available_rect().width() * 0.5)
            .show(ctx, |ui| {
                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                    self.custom_painting(ui);
                })
            });

        // TODO: checking input after drawing imposes 1 frame delay
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
            egui::ComboBox::from_label("Select one!")
                .selected_text(format!("{:?}", self.shader_conf.active_model.as_enum()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.shader_conf.active_model.as_enum(),
                        config::ShadingModelEnum::Phong,
                        "Phong",
                    );
                });
            if self.shader_conf.active_model.build_widget(ui) {
                if let Some(rs) = frame.wgpu_render_state() {
                    self.reload_shader(&rs);
                }
            }
            if self.object.build_widget(ui, ctx) {
                if let Some(rs) = frame.wgpu_render_state() {
                    self.reload_shader(&rs);
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
            renderer::CustomTriangleCallback {
                view_projection: renderer::CameraUniform::from_camera(&self.camera),
                light: self.light,
                model: self.model,
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
