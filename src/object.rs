use eframe::egui_wgpu::wgpu::util::DeviceExt as _;
use eframe::egui_wgpu::wgpu;

const DEFAULT_OBJECT_PATH: &str = "./objects/test1.obj";

use egui_file::FileDialog;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use crate::renderer::ObjectRenderResources;

#[derive(Default)]
pub struct Object {
    pub opened_file: Option<PathBuf>,
    open_file_dialog: Option<FileDialog>,
}

impl Object {
    pub fn load_obj(
        render_state: &egui_wgpu::RenderState,
        path: &Option<&std::path::Path>,
    ) -> (wgpu::Buffer, u32, Option<wgpu::TextureView>, Option<wgpu::Sampler>) {
        let path = path.unwrap_or(std::path::Path::new(DEFAULT_OBJECT_PATH));
        let (models, obj_materials) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )
        .expect("Failed to load OBJ file");

        let mut texture_view = None;
        let mut texture_sampler = None;

        if let Ok(materials) = obj_materials {
            if let Some(mat) = materials.first() {
                if let Some(texture) = &mat.diffuse_texture {
                    eprintln!("{texture:?}");
                    let img = image::open(texture).expect("no image");
                    let rgba = img.to_rgba8();
                    let size = wgpu::Extent3d {
                        width: rgba.width(),
                        height: rgba.height(),
                        depth_or_array_layers: 1,
                    };
                    let texture = render_state.device.create_texture(&wgpu::TextureDescriptor {
                        label: None,
                        size,
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                        view_formats: &[],
                    });
                    render_state.queue.write_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: &texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d::ZERO,
                            aspect: wgpu::TextureAspect::All,
                        },
                        &rgba, 
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(4 * rgba.width()),
                            rows_per_image: Some(rgba.height()),
                        },
                        size
                    );
                    texture_view = Some(texture.create_view(&wgpu::TextureViewDescriptor::default()));
                    texture_sampler = Some(render_state.device.create_sampler(&wgpu::SamplerDescriptor {
                        address_mode_u: wgpu::AddressMode::ClampToEdge,
                        address_mode_v: wgpu::AddressMode::ClampToEdge,
                        address_mode_w: wgpu::AddressMode::ClampToEdge,
                        mag_filter: wgpu::FilterMode::Linear,
                        min_filter: wgpu::FilterMode::Nearest,
                        mipmap_filter: wgpu::FilterMode::Nearest,
                        ..Default::default()
                    }));
                }
            }
        }

        let mesh = &models[0].mesh;

        let mut vertices = Vec::new();
        for i in &mesh.indices {
            let i = *i as usize;
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
            vertices.push(crate::renderer::Vertex::new(position, normal, texcoord));
        }

        let vertex_buffer =
            render_state
                .device
                .create_buffer_init(&eframe::wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&vertices),
                    usage: eframe::wgpu::BufferUsages::VERTEX,
                });
        (vertex_buffer, vertices.len() as u32, texture_view, texture_sampler)
    }

    pub fn update_obj(render_state: &egui_wgpu::RenderState, path: &Option<&std::path::Path>) {
        let path = path.unwrap_or(std::path::Path::new(DEFAULT_OBJECT_PATH));
        let (models, _materials) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )
        .expect("Failed to load OBJ file");
        let mesh = &models[0].mesh;

        let mut vertices = Vec::new();
        for i in &mesh.indices {
            let i = *i as usize;
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
            vertices.push(crate::renderer::Vertex::new(position, normal, texcoord));
        }

        let vertex_buffer =
            render_state
                .device
                .create_buffer_init(&eframe::wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&vertices),
                    usage: eframe::wgpu::BufferUsages::VERTEX,
                });
        render_state
            .renderer
            .write()
            .callback_resources
            .get_mut::<ObjectRenderResources>()
            .unwrap()
            .set_vertex_buffer(vertex_buffer, vertices.len() as u32);
    }

    pub fn build_widget(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) -> bool {
        if (ui.button("Load 3D model")).clicked() {
            // Show only files with the extension "obj".
            let filter = Box::new({
                let ext = Some(OsStr::new("obj"));
                move |path: &Path| -> bool { path.extension() == ext }
            });
            let mut dialog =
                FileDialog::open_file(self.opened_file.clone()).show_files_filter(filter);
            dialog.open();
            self.open_file_dialog = Some(dialog);
        }

        if let Some(dialog) = &mut self.open_file_dialog {
            if dialog.show(ctx).selected() {
                if let Some(file) = dialog.path() {
                    self.opened_file = Some(file.to_path_buf());
                    return true;
                }
            }
        }
        false
    }
}
