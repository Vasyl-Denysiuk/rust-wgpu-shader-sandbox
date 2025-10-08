use egui;
use std::rc::Rc;
struct ShaderConfig {
    active_model: Box<dyn ShadingModel>,
    active_effects: Vec<Rc<dyn AfterEffects>>,
}

trait ShadingModel {
    fn build_widget(&self, ui: &mut egui::Ui) -> Box<dyn egui::Widget>;
}

trait AfterEffects {
    fn build_widget(&self) -> Box<dyn egui::Widget>;
}

struct Phong {
    amb_coef: f32,
    diff_coef: f32,
    spec_coef: f32,
    shininess: f32,
}

impl Phong {
    fn new() -> Phong {
        Phong {
            amb_coef: 0.05,
            diff_coef: 0.5,
            spec_coef: 0.5,
            shininess: 4.0
        }
    }
}

impl ShadingModel for Phong {
    fn build_widget(&self, ui: &mut egui::Ui) -> Box<dyn egui::Widget> {
        todo!();
    }
}