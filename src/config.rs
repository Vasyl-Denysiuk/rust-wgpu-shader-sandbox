pub mod phong;
use std::sync::{Arc, Mutex};

pub struct ShaderConfig {
    pub active_model: Arc<Mutex<dyn ShadingModel + Send>>,
}

pub trait ShadingModel {
    fn build_widget(&mut self, ui: &mut egui::Ui) -> bool;
    fn get_source(&self) -> String;
    fn as_enum(&self) -> ShadingModelEnum;
    fn create_uniform(
        &self,
        device: &eframe::wgpu::Device,
    ) -> (
        eframe::wgpu::BindGroupLayout,
        eframe::wgpu::BindGroup,
        eframe::wgpu::Buffer,
    );
    fn to_params(&self) -> &[u8];
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ShadingModelEnum {
    Phong,
}
