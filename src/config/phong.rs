use crate::config::{ShadingModel, ShadingModelEnum};

pub struct Phong {
    ka: f32,
    kd: f32,
    ks: f32,
    alph: f32,
}

impl Phong {
    pub fn new() -> Phong {
        Phong {
            ka: 0.05,
            kd: 0.4,
            ks: 0.4,
            alph: 4.0,
        }
    }
}

impl ShadingModel for Phong {
    fn build_widget(&mut self, ui: &mut egui::Ui) -> bool {
        ui.vertical(|ui| {
            let mut should_update = false;
            ui.style_mut().spacing.slider_width = ui.available_width();

            ui.label(format!("ambient strength: {}", self.ka));
            should_update |= ui
                .add(egui::Slider::new(&mut self.ka, 0.0..=1.0))
                .drag_stopped();

            ui.label(format!("diffuse strength: {}", self.kd));
            should_update |= ui
                .add(egui::Slider::new(&mut self.kd, 0.0..=1.0))
                .drag_stopped();

            ui.label(format!("specular strength: {}", self.ks));
            should_update |= ui
                .add(egui::Slider::new(&mut self.ks, 0.0..=1.0))
                .drag_stopped();

            ui.label(format!("shininess: {}", self.alph));
            should_update |= ui
                .add(egui::Slider::new(&mut self.alph, 0.0..=100.0))
                .drag_stopped();

            should_update
        })
        .inner
    }

    fn get_source(&self) -> String {
        let _path: std::path::PathBuf = ["shaders", "phong.wgsl"].iter().collect();
        include_str!("shaders/phong.wgsl").into()
    }

    fn get_constants(&self) -> Vec<(&str, f64)> {
        vec![
            ("ka", self.ka as f64),
            ("kd", self.kd as f64),
            ("ks", self.ks as f64),
            ("alph", self.alph as f64),
        ]
    }

    fn as_enum(&self) -> ShadingModelEnum {
        ShadingModelEnum::Phong
    }
}
