use egui;
pub mod phong;

pub struct ShaderConfig {
    pub active_model: Box<dyn ShadingModel>,
}

pub trait ShadingModel {
    fn build_widget(&mut self, ui: &mut egui::Ui) -> bool;
    fn get_source(&self) -> String;
    fn get_constants(&self) -> Vec<(&str, f64)>;
    fn as_enum(&self) -> ShadingModelEnum;
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ShadingModelEnum {
    Phong,
}
