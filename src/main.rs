pub mod config;
pub mod renderer;
pub mod camera;
const DEFAULT_MODEL_PATH: &str = "./test.obj";
const DEFAULT_SHADER: &str = include_str!("./shaders/phong.wgsl");

use eframe::egui_wgpu;

pub struct App {
    shader_conf: ShaderConfig,
    camera: camera::WorldCamera,
    light: renderer::LightUniform,
    model: renderer::ModelUniform,
}

pub struct ShaderConfig {
    shader_src: String,
    obj_path: String,
}

impl App {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Option<Self> {
        let wgpu_render_state = cc.wgpu_render_state.as_ref()?;
        renderer::build_pipeline(
            wgpu_render_state,
            None,
             None,
            &[]
        );

        Some(Self {
            camera: camera::WorldCamera::new(),
            light: renderer::LightUniform::new(),
            model: renderer::ModelUniform::new(),
            shader_conf: ShaderConfig{
                shader_src: DEFAULT_SHADER.into(),
                obj_path: DEFAULT_MODEL_PATH.into()
            }
            })
    }

    fn reload_shader(&self, render_state: &egui_wgpu::RenderState) {
        renderer::build_pipeline(
            render_state, Some(self.shader_conf.shader_src.clone()),
            Some(self.shader_conf.obj_path.clone()),
            &[]
        );
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.camera.resize(0.5*ctx.available_rect().width()/ctx.available_rect().height());

        let viewport_response = egui::SidePanel::left("viewport_panel")
            .exact_width(ctx.available_rect().width() * 0.5)
            .show(ctx, |ui| {
                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                    self.custom_painting(ui);
                });
            });

        if viewport_response.response.hovered() {
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
            if ctx.input(|i| i.key_down(egui::Key::Enter)) {
                self.camera.down();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Enter new WGSL shader code:");
            egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut self.shader_conf.shader_src)
                    .code_editor()
                    .desired_rows(10)
                    .desired_width(f32::INFINITY)
                );
            if ui.button("Compile Shader").clicked() {
                if let Some(rs) = frame.wgpu_render_state() {
                    self.reload_shader(&rs);
                }
            }
            });

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
    eframe::run_native("egui wgpu demo", nativeoptions, Box::new(|cc| Ok(Box::new(App::new(cc).unwrap())))).unwrap();
}
