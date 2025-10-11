use eframe::egui_wgpu::wgpu::util::DeviceExt as _;

const DEFAULT_OBJECT_PATH: &str = "./test1.obj";

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
    ) -> (eframe::wgpu::Buffer, u32) {
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
        (vertex_buffer, vertices.len() as u32)
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
