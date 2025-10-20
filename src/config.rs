pub mod flat;
pub mod phong;
use std::sync::{Arc, Mutex};

pub struct ShaderConfig {
    pub active_model: Arc<Mutex<dyn ShadingModel + Send>>,
    pub active_post_effects: Vec<Arc<Mutex<dyn PostEffect + Send>>>
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

pub trait PostEffect {
    fn build_widget(&mut self, ui: &mut egui::Ui) -> bool;
    fn get_source(&self) -> String;
    fn as_enum(&self) -> PostEffectEnum;
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
    Flat,
}

pub enum PostEffectEnum {
    Negative,
    ChromaticAberration,
    Blur,
}
