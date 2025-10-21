use eframe::{
    egui_wgpu::{self, wgpu},
    wgpu::util::DeviceExt,
};
use std::sync::{Arc, Mutex};

use wgpu::PipelineCompilationOptions;

use crate::config::{self, PostEffect, ShadingModel};

pub fn build_pipeline(
    render_state: &egui_wgpu::RenderState,
    path: &Option<&std::path::Path>,
    shading_model: &(impl config::ShadingModel + ?Sized),
) {
    let src = shading_model.get_source();
    let device = &render_state.device;
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(src.into()),
    });

    let (camera_bind_group_layout, camera_bind_group, camera_buffer) =
        CameraUniform::create_uniform(device);

    let (light_bind_group_layout, light_bind_group, light_buffer) =
        LightUniform::create_uniform(device);

    let (params_bind_group_layout, params_bind_group, params_buffer) =
        shading_model.create_uniform(device);

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[
            &camera_bind_group_layout,
            &light_bind_group_layout,
            &params_bind_group_layout,
        ],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[Vertex::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(render_state.target_format.into())],
            compilation_options: PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Cw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    let (vertex_buffer, vertex_count) = crate::object::Object::load_obj(render_state, path);

    crate::object::Object::load_obj(render_state, path);
    render_state
        .renderer
        .write()
        .callback_resources
        .insert(ObjectRenderResources {
            pipeline,
            camera_bind_group,
            camera_buffer,
            light_buffer,
            light_bind_group,
            params_buffer,
            params_bind_group,
            vertex_buffer,
            vertex_count,
            post_process_resources: None,
        });
}

pub struct PostProcessResources {
    depth_texture_view: wgpu::TextureView,
    _depth_sampler: wgpu::Sampler,
    target_format: wgpu::TextureFormat,
    texture_view_a: wgpu::TextureView,
    texture_view_b: wgpu::TextureView,
    bind_group_a: wgpu::BindGroup,
    bind_group_b: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    out: PostProcessTexture,
}

#[derive(Clone, Copy)]
enum PostProcessTexture {
    A,
    B,
}

impl PostProcessResources {
    fn swap_buffers(&mut self) {
        self.out = match self.out {
            PostProcessTexture::A => PostProcessTexture::B,
            PostProcessTexture::B => PostProcessTexture::A,
        };
    }
    
    fn get_texture_out_view(&self) -> &wgpu::TextureView {
        match self.out {
            PostProcessTexture::A => &self.texture_view_a,
            PostProcessTexture::B => &self.texture_view_b,
        }
    }
    
    fn get_bind_group_in(&self) -> &wgpu::BindGroup {
        match self.out {
            PostProcessTexture::A => &self.bind_group_b,
            PostProcessTexture::B => &self.bind_group_a,
        }
    }

    pub fn get_final_bind_group(&self) -> &wgpu::BindGroup {
        match self.out {
            PostProcessTexture::A => &self.bind_group_a,
            PostProcessTexture::B => &self.bind_group_b,
        }
    }
}

pub fn post_effect_init(render_state: &egui_wgpu::RenderState, size: (u32, u32)) {
    let device = &render_state.device;
    let target_format = render_state.target_format;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(include_str!("post.wgsl").into()),
    });

    let tex_desc = &wgpu::TextureDescriptor {
        label: Some("post_process_texture"),
        size: wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: target_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[target_format],
    };

    let depth_desc = &wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };

    let texture_a = device.create_texture(tex_desc);
    let texture_b = device.create_texture(tex_desc);
    let depth_texture = device.create_texture(depth_desc);

    let depth_texture_view = depth_texture.create_view(&Default::default());
    let texture_view_a = texture_a.create_view(&Default::default());
    let texture_view_b = texture_b.create_view(&Default::default());

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let _depth_sampler = device.create_sampler(
        &wgpu::SamplerDescriptor { // 4.
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual), // 5.
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        }
    );

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: None,
    });

    let bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view_a),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
        label: None,
    });

    let bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view_b),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
        label: None,
    });

    let quad_vertices: &[PostVertex] = &[
        PostVertex::new([-1.0, 1.0], [0.0, 0.0]),
        PostVertex::new([1.0, 1.0], [1.0, 0.0]),
        PostVertex::new([1.0, -1.0], [1.0, 1.0]),
        PostVertex::new([1.0, -1.0], [1.0, 1.0]),
        PostVertex::new([-1.0, -1.0], [0.0, 1.0]),
        PostVertex::new([-1.0, 1.0], [0.0, 0.0]),
    ];

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("fullscreen_quad"),
        contents: bytemuck::cast_slice(quad_vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[PostVertex::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(target_format.into())],
            compilation_options: PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Cw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    let post_process_resources = PostProcessResources {
        depth_texture_view,
        _depth_sampler,
        target_format,
        texture_view_a,
        texture_view_b,
        bind_group_a,
        bind_group_b,
        pipeline,
        vertex_buffer,
        out: PostProcessTexture::A,
    };

    render_state
        .renderer
        .write()
        .callback_resources
        .get_mut::<ObjectRenderResources>()
        .unwrap()
        .set_post_process_resources(post_process_resources);
}

pub fn create_post_pipeline(device: &wgpu::Device, target_format: wgpu::TextureFormat, src: String) ->wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(src.into()),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: None,
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[PostVertex::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(target_format.into())],
            compilation_options: PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Cw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}

pub struct ObjectRenderCallback {
    pub view_projection: CameraUniform,
    pub light: LightUniform,
    pub shading_model: Arc<Mutex<dyn ShadingModel + Send>>,
    pub post_effects: Vec<Arc<Mutex<dyn PostEffect + Send>>>,
}

impl egui_wgpu::CallbackTrait for ObjectRenderCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources = resources.get_mut::<ObjectRenderResources>().unwrap();
        resources.prepare(
            device,
            queue,
            &self.view_projection,
            &self.light,
            self.shading_model.clone(),
        );
        if let Some(post) = &mut resources.post_process_resources {
            let mut encoder = device.create_command_encoder(&Default::default());
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &post.get_texture_out_view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &post.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&resources.pipeline);
            pass.set_bind_group(0, &resources.camera_bind_group, &[]);
            pass.set_bind_group(1, &resources.light_bind_group, &[]);
            pass.set_bind_group(2, &resources.params_bind_group, &[]);
            pass.set_vertex_buffer(0, resources.vertex_buffer.slice(..));
            pass.draw(0..resources.vertex_count, 0..1);
            drop(pass);
            
            for post_effect in self.post_effects.iter() {
                let mut post_guard = post_effect.lock().unwrap();
                let post_effect_pipeline = post_guard.get_pipeline(device, post.target_format);
                post.swap_buffers();
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: post.get_texture_out_view(),
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(post_effect_pipeline);
                pass.set_bind_group(0, post.get_bind_group_in(), &[]);
                pass.set_vertex_buffer(0, post.vertex_buffer.slice(..));
                pass.draw(0..6, 0..1);
            }
            queue.submit([encoder.finish()]);
        }
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
        let resources: &ObjectRenderResources = resources.get().unwrap();
        resources.paint(render_pass);
    }
}

pub struct ObjectRenderResources {
    pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    params_buffer: wgpu::Buffer,
    params_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
    post_process_resources: Option<PostProcessResources>,
}

impl ObjectRenderResources {
    pub fn set_vertex_buffer(&mut self, vertex_buffer: wgpu::Buffer, vertex_count: u32) {
        self.vertex_buffer = vertex_buffer;
        self.vertex_count = vertex_count;
    }

    pub fn set_post_process_resources(&mut self, post_process_resources: PostProcessResources) {
        self.post_process_resources = Some(post_process_resources);
    }

    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        view_projection: &CameraUniform,
        light: &LightUniform,
        params: Arc<Mutex<dyn ShadingModel + Send>>,
    ) {
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[*view_projection]));
        queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[*light]));
        queue.write_buffer(&self.params_buffer, 0, params.lock().unwrap().to_params());
    }

    fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        if let Some(pp) = &self.post_process_resources {
            render_pass.set_pipeline(&pp.pipeline);
            render_pass.set_bind_group(0, &pp.bind_group_a, &[]);
            render_pass.set_vertex_buffer(0, pp.vertex_buffer.slice(..));
            render_pass.draw(0..6, 0..1);
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub proj: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn from_camera(camera: &crate::camera::WorldCamera) -> Self {
        Self {
            proj: camera.build_projection().to_cols_array_2d(),
            view: camera.build_view().to_cols_array_2d(),
        }
    }

    fn create_uniform(
        device: &wgpu::Device,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup, wgpu::Buffer) {
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(
                            wgpu::BufferSize::new(std::mem::size_of::<CameraUniform>() as u64)
                                .unwrap(),
                        ),
                    },
                    count: None,
                }],
            });

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        (camera_bind_group_layout, camera_bind_group, camera_buffer)
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PostVertex {
    position: [f32; 2],
    texcoord: [f32; 2],
}

impl PostVertex {
    pub fn new(position: [f32; 2], texcoord: [f32; 2]) -> PostVertex {
        PostVertex { position, texcoord }
    }
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<PostVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
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
    pub fn new(position: [f32; 3], normal: [f32; 3], texcoord: [f32; 2]) -> Vertex {
        Vertex {
            position,
            normal,
            texcoord,
        }
    }
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

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    position: [f32; 3],
    _padding: u32,
    color: [f32; 3],
    _padding2: u32,
}

impl LightUniform {
    pub fn new() -> LightUniform {
        LightUniform {
            position: [0.0, 3.0, -3.0],
            _padding: 0,
            color: [1., 1., 1.],
            _padding2: 0,
        }
    }

    fn create_uniform(
        device: &wgpu::Device,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup, wgpu::Buffer) {
        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<LightUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });
        (light_bind_group_layout, light_bind_group, light_buffer)
    }
}