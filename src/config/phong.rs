use crate::config::{ShadingModel, ShadingModelEnum};
use eframe::egui_wgpu::wgpu;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
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

    fn as_enum(&self) -> ShadingModelEnum {
        ShadingModelEnum::Phong
    }

    fn create_uniform(
        &self,
        device: &wgpu::Device,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup, wgpu::Buffer) {
        let phong_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(
                            wgpu::BufferSize::new(std::mem::size_of::<Phong>() as u64).unwrap(),
                        ),
                    },
                    count: None,
                }],
            });

        let phong_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<Phong>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let phong_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("phong"),
            layout: &phong_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: phong_buffer.as_entire_binding(),
            }],
        });

        (phong_bind_group_layout, phong_bind_group, phong_buffer)
    }

    fn to_params(&self) -> &[u8] {
        bytemuck::cast_ref::<_, [u8; size_of::<Phong>()]>(self)
    }
}
