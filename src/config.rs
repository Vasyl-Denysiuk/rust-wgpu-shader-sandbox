pub mod blur;
pub mod chromatic;
pub mod flat;
pub mod negative;
pub mod phong;
use std::sync::{Arc, Mutex};

pub struct ShaderConfig {
    pub active_model: Arc<Mutex<dyn ShadingModel + Send>>,
    pub active_post_effects: Vec<Arc<Mutex<dyn PostEffect + Send>>>,
    pub selected_effect: Option<PostEffectEnum>,
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
    fn get_source(&self) -> String;
    fn get_pipeline(
        &mut self,
        device: &eframe::wgpu::Device,
        target_format: eframe::wgpu::TextureFormat,
    ) -> &egui_wgpu::wgpu::RenderPipeline;
    fn as_enum(&self) -> PostEffectEnum;
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ShadingModelEnum {
    Phong,
    Flat,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum PostEffectEnum {
    Negative,
    ChromaticAberration,
    Blur,
}

impl std::fmt::Display for PostEffectEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
