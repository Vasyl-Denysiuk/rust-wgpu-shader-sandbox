use crate::{config::PostEffect, renderer};

pub struct Blur {
    pipeline: Option<egui_wgpu::wgpu::RenderPipeline>,
}

impl PostEffect for Blur {
    fn get_source(&self) -> String {
        todo!()
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

    fn as_enum(&self) -> super::PostEffectEnum {
        super::PostEffectEnum::Blur
    }
}

impl Blur {
    pub fn new() -> Blur {
        Blur { pipeline: None }
    }
}
