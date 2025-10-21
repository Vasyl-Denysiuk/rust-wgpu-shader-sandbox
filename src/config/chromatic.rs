use crate::{config::PostEffect, renderer};

pub struct ChromaticAberration {
    pipeline: Option<egui_wgpu::wgpu::RenderPipeline>
}

impl PostEffect for ChromaticAberration {
    fn get_source(&self) -> String {
        let _path: std::path::PathBuf = ["post", "chromatic.wgsl"].iter().collect();
        include_str!("post/chromatic.wgsl").into()
    }

    fn get_pipeline(&mut self, device: &eframe::wgpu::Device, target_format: eframe::wgpu::TextureFormat) -> &egui_wgpu::wgpu::RenderPipeline {
        if self.pipeline.is_none() {
            self.pipeline = Some(renderer::create_post_pipeline(device, target_format, self.get_source()));
        }
        &self.pipeline.as_ref().unwrap()
    }

    fn as_enum(&self) -> super::PostEffectEnum {
        super::PostEffectEnum::ChromaticAberration
    }
}

impl ChromaticAberration {
    pub fn new() -> ChromaticAberration {
        ChromaticAberration {pipeline: None}
    }
}