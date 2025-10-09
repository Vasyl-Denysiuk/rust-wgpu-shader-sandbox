use egui;
use std::rc::Rc;
struct ShaderConfig {
    active_model: Box<dyn ShadingModel>,
    active_effects: Vec<Rc<dyn AfterEffects>>,
}

trait ShadingModel {
    fn build_widget(&self, ui: &mut egui::Ui) -> Box<dyn egui::Widget>;
    fn build_source(&self) -> String;
}

trait AfterEffects {
    fn build_widget(&self) -> Box<dyn egui::Widget>;
}

struct Phong {
    ka: f32,
    kd: f32,
    ks: f32,
    alph: f32,
}

impl Phong {
    fn new() -> Phong {
        Phong {
            ka: 0.05,
            kd: 0.4,
            ks: 0.4,
            alph: 4.0
        }
    }
}

impl ShadingModel for Phong {
    fn build_widget(&self, ui: &mut egui::Ui) -> Box<dyn egui::Widget> {
        todo!();
    }

    fn build_source(&self) -> String {
        todo!();
    }
}