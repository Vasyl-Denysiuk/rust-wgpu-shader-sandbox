use egui;
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
            let mut has_changed = false;
            ui.style_mut().spacing.slider_width = ui.available_width();
            ui.label("ambient strength");
            has_changed |= ui.add(egui::Slider::new(&mut self.ka, 0.0..=1.0)).changed();
            ui.label("diffuse strength");
            has_changed |= ui.add(egui::Slider::new(&mut self.kd, 0.0..=1.0)).changed();
            ui.label("specular strength");
            has_changed |= ui.add(egui::Slider::new(&mut self.ks, 0.0..=1.0)).changed();
            ui.label("shininess");
            has_changed |= ui
                .add(egui::Slider::new(&mut self.alph, 0.0..=100.0))
                .changed();
            has_changed
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
