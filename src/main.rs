const DEFAULT_MODEL_PATH: &str = "./model.obj";
const DEFAULT_SHADER: &str =
"
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) texcoord: vec2<f32>,
};
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) texcoord: vec2<f32>,
};
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.normal = in.normal;
    out.texcoord = in.texcoord;
    return out;
}
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = vec3<f32>(0.0);
    return vec4<f32>(color, 1.0);
}
";

use eframe::egui_wgpu::{
    self,
    wgpu::{
        self,
        util::DeviceExt as _
    }
};

pub struct App {
    angle: f32,
    shader_conf: ShaderConfig,
    camera: WorldCamera,
}

pub struct ShaderConfig {
    shader_src: String,
    obj_path: String,
}

impl App {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Option<Self> {
        let wgpu_render_state = cc.wgpu_render_state.as_ref()?;
        Self::build_pipeline(wgpu_render_state, None, None);

        Some(Self {
            camera: WorldCamera::new(),
            angle: 0.0,
            shader_conf: ShaderConfig{
                shader_src: DEFAULT_SHADER.into(),
                obj_path: DEFAULT_MODEL_PATH.into()
            }
            })
    }

    fn build_pipeline(render_state: &egui_wgpu::RenderState, shader_src: Option<String>, obj_path: Option<String>) {
        let src = shader_src.as_deref().unwrap_or(DEFAULT_SHADER);
        let device = &render_state.device;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(src.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(wgpu::BufferSize::new(std::mem::size_of::<CameraUniform>() as u64).unwrap()),
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: None,
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(render_state.target_format.into())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let uniform_buffer = render_state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera_uniform"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("custom3d"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let (vertex_buffer, vertex_count) = Self::load_obj(render_state, obj_path);
        render_state
            .renderer
            .write()
            .callback_resources
            .insert(TriangleRenderResources {
                pipeline,
                bind_group,
                uniform_buffer,
                vertex_buffer,
                vertex_count
            });
    }

    fn reload_shader(&self, render_state: &egui_wgpu::RenderState, shader_src: String) {
        Self::build_pipeline(render_state, Some(shader_src), Some(self.shader_conf.obj_path.clone()));
    }

    fn load_obj(render_state: &egui_wgpu::RenderState, path: Option<String>) -> (wgpu::Buffer, u32) {
        let path = path.as_deref().unwrap_or(DEFAULT_MODEL_PATH);
        let (models, _materials) = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS).expect("Failed to load OBJ file");
        let mesh = &models[0].mesh;

        let mut vertices = Vec::new();
        for i in 0..mesh.positions.len() / 3 {
            let position = [
                mesh.positions[i * 3],
                mesh.positions[i * 3 + 1],
                mesh.positions[i * 3 + 2],
            ];
            let normal = if !mesh.normals.is_empty() {
                [
                    mesh.normals[i * 3],
                    mesh.normals[i * 3 + 1],
                    mesh.normals[i * 3 + 2],
                ]
            } else {
                [0.0, 0.0, 1.0]
            };
            let texcoord = if !mesh.texcoords.is_empty() {
                [mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]]
            } else {
                [0.0, 0.0]
            };
            vertices.push(Vertex { position, normal, texcoord });
        }

        let vertex_buffer = render_state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex_buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        (vertex_buffer, vertices.len() as u32)
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        const CAM_SPEED: f32 = 0.03;
        egui::SidePanel::left("viewport_panel")
            .exact_width(ctx.available_rect().width() * 0.5)
            .show(ctx, |ui| {
                if ctx.input(|i| i.key_down(egui::Key::W)) {
                    self.camera.position += self.camera.forward() * CAM_SPEED;
                }
                if ctx.input(|i| i.key_down(egui::Key::S)) {
                    self.camera.position -= self.camera.forward() * CAM_SPEED;
                }
                if ctx.input(|i| i.key_down(egui::Key::D)) {
                    self.camera.position += self.camera.right() * CAM_SPEED;
                }
                if ctx.input(|i| i.key_down(egui::Key::A)) {
                    self.camera.position -= self.camera.right() * CAM_SPEED;
                }

                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                    self.custom_painting(ui);
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Enter new WGSL shader code:");
            ui.text_edit_multiline(&mut self.shader_conf.shader_src);

            if ui.button("Compile Shader").clicked() {
                if let Some(rs) = frame.wgpu_render_state() {
                    self.reload_shader(&rs, self.shader_conf.shader_src.clone());
                }
            }
        });
        ctx.request_repaint();
    }
}
struct CustomTriangleCallback {
    view_projection: CameraUniform,
}

impl egui_wgpu::CallbackTrait for CustomTriangleCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources: &TriangleRenderResources = resources.get().unwrap();
        resources.prepare(device, queue, &self.view_projection);
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
        let resources: &TriangleRenderResources = resources.get().unwrap();
        resources.paint(render_pass);
    }
}

impl App {
    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::drag());

        self.angle += response.drag_motion().x * 0.01;
        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            CustomTriangleCallback { view_projection: CameraUniform::from_camera(&self.camera) },
        ));
    }
}

struct TriangleRenderResources {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
}

impl TriangleRenderResources {
    fn prepare(&self, _device: &wgpu::Device, queue: &wgpu::Queue, view_projection: &CameraUniform) {
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[*view_projection]),
        );
    }

    fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.vertex_count, 0..1);
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn from_camera(camera: &WorldCamera) -> Self {
        Self {
            view_proj: camera.build_view_projection().to_cols_array_2d(),
        }
    }
}

pub struct WorldCamera {
    position: glam::Vec3,
    rotation: glam::Vec3,
    aspect: f32,
    fovy: f32,
    z_near: f32,
    z_far: f32,
}

impl WorldCamera {
    fn new() -> WorldCamera {
        WorldCamera {
            position: glam::vec3(0.0, 0.0, 3.0),
            rotation: glam::vec3(0.0, 0.0, 0.0),
            aspect: 1.,
            fovy: 45f32.to_radians(),
            z_near: 0.1,
            z_far: 100.0,
        }
    }

    fn build_view_projection(&self) -> glam::Mat4 {
        let rotation_matrix = glam::Mat4::from_euler(
            glam::EulerRot::ZYX, 
            self.rotation.z, 
            self.rotation.y, 
            self.rotation.x
        );
        let translation_matrix = glam::Mat4::from_translation(-self.position);

        let view = rotation_matrix * translation_matrix;
        let proj = glam::Mat4::perspective_rh(self.fovy, self.aspect, self.z_near, self.z_far);

        proj * view
    }
    pub fn forward(&self) -> glam::Vec3 {
        let rot = glam::Quat::from_euler(glam::EulerRot::ZYX, self.rotation.z, self.rotation.y, self.rotation.x);
        rot * glam::Vec3::new(0.0, 0.0, -1.0)
    }

    pub fn right(&self) -> glam::Vec3 {
        let rot = glam::Quat::from_euler(glam::EulerRot::ZYX, self.rotation.z, self.rotation.y, self.rotation.x);
        rot * glam::Vec3::new(1.0, 0.0, 0.0)
    }

    pub fn up(&self) -> glam::Vec3 {
        let rot = glam::Quat::from_euler(glam::EulerRot::ZYX, self.rotation.z, self.rotation.y, self.rotation.x);
        rot * glam::Vec3::new(0.0, 1.0, 0.0)
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    texcoord: [f32; 2],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() * 2) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

fn main() {
    let nativeoptions = eframe::NativeOptions::default();
    eframe::run_native("egui wgpu demo", nativeoptions, Box::new(|cc| Ok(Box::new(App::new(cc).unwrap())))).unwrap();
}