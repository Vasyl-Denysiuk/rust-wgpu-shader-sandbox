use crate::{config::PostEffect, renderer};

pub struct Negative {
    pipeline: Option<egui_wgpu::wgpu::RenderPipeline>,
}

impl PostEffect for Negative {
    fn get_source(&self) -> String {
        let _path: std::path::PathBuf = ["post", "negative.wgsl"].iter().collect();
        include_str!("post/negative.wgsl").into()
    }

    fn as_enum(&self) -> super::PostEffectEnum {
        super::PostEffectEnum::Negative
    }

    fn get_pipeline(
        &mut self,
        device: &eframe::wgpu::Device,
        target_format: eframe::wgpu::TextureFormat,
    ) -> &egui_wgpu::wgpu::RenderPipeline {
        if self.pipeline.is_none() {
            self.pipeline = Some(renderer::create_post_pipeline(
                device,
                target_format,
                self.get_source(),
            ));
        }
        self.pipeline.as_ref().unwrap()
    }
}

impl Negative {
    pub fn new() -> Negative {
        Negative { pipeline: None }
    }
}
