use core::f32;
use std::path::Path;

use bytemuck::{Pod, Zeroable};
use cgmath::{Matrix, SquareMatrix};
use gltf::Node;
use wgpu::util::DeviceExt;

use crate::app::{buffer::BindGroupBuilder, texture::Texture2d, vertex::Vertex, vertex::Transform};

pub struct Mesh {
    pub name: String,
    pub vert_buffer: Vec<Vertex>,
    pub inst_buffer: Vec<Transform>,
    pub ind_buffer: Vec<u32>,
}

impl Mesh {
    pub fn build(&self, device: &wgpu::Device) -> BuiltMesh {
        let vert_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&(self.name.clone() + "Vertex Buffer")),
            contents: bytemuck::cast_slice(&self.vert_buffer),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let ind_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&(self.name.clone() + "Index Buffer")),
            contents: bytemuck::cast_slice(&self.ind_buffer),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        let inst_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&(self.name.clone() + " Inst Buffer")),
            contents: bytemuck::cast_slice(&self.inst_buffer),
            usage: wgpu::BufferUsages::VERTEX,
        });

        BuiltMesh {
            name: self.name.clone(),
            vert_buffer, inst_buffer, ind_buffer,
            items: self.ind_buffer.len() as u32,
            instances: self.inst_buffer.len() as u32,
        }
    }
}

pub struct BuiltMesh {
    pub name: String,
    pub vert_buffer: wgpu::Buffer,
    pub inst_buffer: wgpu::Buffer,
    pub ind_buffer: wgpu::Buffer,
    pub items: u32,
    pub instances: u32,
}

pub struct Material {
    pub name: String,
    pub buffer: wgpu::Buffer,
    pub buffer_bg: wgpu::BindGroup,
    pub diffuse_tex: Option<Texture2d> = None,
    pub normal_tex: Option<Texture2d> = None,
    pub mesh_data: Vec<Mesh> = vec![],
    pub meshes: Vec<BuiltMesh> = vec![],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct MaterialBuffer {
    pub diffuse_color: [f32; 4],
    pub normals: u32,
    pad_1: [u32; 3],
    pub emissive_color: [f32; 3],
    pad_2: u32,
    pub mr_base: [f32; 2],
    pad_3: [u32; 2],
    pub diffuse_transform: [[f32; 4]; 3],
}

fn get_mat3(mat4: &cgmath::Matrix4<f32>) -> cgmath::Matrix3<f32> {
    cgmath::Matrix3::from([
        [mat4.x.x, mat4.x.y, mat4.x.z],
        [mat4.y.x, mat4.y.y, mat4.y.z],
        [mat4.z.x, mat4.z.y, mat4.z.z]
    ])
}

const fn from_mat3(mat3: &cgmath::Matrix3<f32>) -> [[f32; 4]; 3] {
    [
        [mat3.x.x, mat3.x.y, mat3.x.z, 0.0],
        [mat3.y.x, mat3.y.y, mat3.y.z, 0.0],
        [mat3.z.x, mat3.z.y, mat3.z.z, 0.0]
    ]
}

impl Default for MaterialBuffer {
    fn default() -> Self {
        use cgmath::SquareMatrix;
        Self {
            diffuse_color: [0.0; 4],
            normals: 0_u32,
            pad_1: [0_u32; 3],
            emissive_color: [0.0; 3],
            pad_2: 0_u32,
            mr_base: [0.0; 2],
            pad_3: [0_u32; 2],
            diffuse_transform: from_mat3(&cgmath::Matrix3::identity()),
        }
    }
}

pub struct Model {
    pub materials: Vec<Material>,
    pub mesh_data: Vec<Mesh>,
    pub meshes: Vec<BuiltMesh>,
}

impl Model {
    #[inline(always)]
    fn get_texture(tex_info: gltf::texture::Info, buffers: &[gltf::buffer::Data], name: &str, device: &wgpu::Device, queue: &wgpu::Queue, tex_layout: &wgpu::BindGroupLayout) -> anyhow::Result<Texture2d> {
        let tex = tex_info.texture();
        let s_wrap = match tex.sampler().wrap_s() {
            gltf::texture::WrappingMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
            gltf::texture::WrappingMode::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
            gltf::texture::WrappingMode::Repeat => wgpu::AddressMode::Repeat,
        };
        let t_wrap = match tex.sampler().wrap_t() {
            gltf::texture::WrappingMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
            gltf::texture::WrappingMode::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
            gltf::texture::WrappingMode::Repeat => wgpu::AddressMode::Repeat,
        };
        let tex_name = name.to_string() + " Diffuse Texture";
        let tex_name = tex.name().unwrap_or(&tex_name);
        let tex_src = tex.source().source();
        match tex_src {
            gltf::image::Source::View { view, mime_type: _ } => {
                let mut tex = Texture2d::from_bytes(
                    device,
                    queue, 
                    &buffers[view.buffer().index()][view.offset()..view.offset()+view.length()],
                    tex_name,
                    (s_wrap, t_wrap)
                )?;
                tex.create_bind_group(tex_name.to_string(), device, tex_layout);
                Ok(tex)
            },
            gltf::image::Source::Uri { uri, mime_type: _ } => {
                let mut tex = Texture2d::load_texture(uri, device, queue, (s_wrap, t_wrap))?;
                tex.create_bind_group(tex_name.to_string(), device, tex_layout);
                Ok(tex)
            }
        }
    }

    #[inline(always)]
    fn get_normal_texture(tex_info: gltf::material::NormalTexture, buffers: &[gltf::buffer::Data], name: &str, device: &wgpu::Device, queue: &wgpu::Queue, tex_layout: &wgpu::BindGroupLayout) -> anyhow::Result<Texture2d> {
        let tex = tex_info.texture();
        let s_wrap = match tex.sampler().wrap_s() {
            gltf::texture::WrappingMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
            gltf::texture::WrappingMode::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
            gltf::texture::WrappingMode::Repeat => wgpu::AddressMode::Repeat,
        };
        let t_wrap = match tex.sampler().wrap_t() {
            gltf::texture::WrappingMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
            gltf::texture::WrappingMode::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
            gltf::texture::WrappingMode::Repeat => wgpu::AddressMode::Repeat,
        };
        let tex_name = name.to_string() + " Diffuse Texture";
        let tex_name = tex.name().unwrap_or(&tex_name);
        let tex_src = tex.source().source();
        match tex_src {
            gltf::image::Source::View { view, mime_type: _ } => {
                let mut tex = Texture2d::from_bytes(
                    device,
                    queue, 
                    &buffers[view.buffer().index()][view.offset()..view.offset()+view.length()],
                    tex_name,
                    (s_wrap, t_wrap)
                )?;
                tex.create_bind_group(tex_name.to_string(), device, tex_layout);
                Ok(tex)
            },
            gltf::image::Source::Uri { uri, mime_type: _ } => {
                let mut tex = Texture2d::load_texture(uri, device, queue, (s_wrap, t_wrap))?;
                tex.create_bind_group(tex_name.to_string(), device, tex_layout);
                Ok(tex)
            }
        }
    }

    fn load_materials(&mut self, document: &gltf::Document, buffers: &[gltf::buffer::Data], device: &wgpu::Device, queue: &wgpu::Queue,
      tex_layout: &wgpu::BindGroupLayout, mat_layout: &wgpu::BindGroupLayout) -> anyhow::Result<()> {
        for m in document.materials() {
            let name = format!("Material {}", self.materials.len());
            let name = m.name().unwrap_or(&name);
            let pbr = m.pbr_metallic_roughness();
            let mut material_buffer_data = MaterialBuffer {
                diffuse_color: pbr.base_color_factor(),
                normals: 0_u32,
                emissive_color: m.emissive_factor(),
                mr_base: [pbr.metallic_factor(), pbr.roughness_factor()],
                ..Default::default()
            };
            let diffuse_tex = if let Some(tex_info) = pbr.base_color_texture() {
                material_buffer_data.diffuse_color[0] = -1.0;
                if let Some(t) = tex_info.texture_transform() {
                    material_buffer_data.diffuse_transform = from_mat3(&(
                        cgmath::Matrix3::from_translation(t.offset().into()) *
                        cgmath::Matrix3::from_angle_z(cgmath::Rad(t.rotation())) * 
                        cgmath::Matrix3::from_nonuniform_scale(t.scale()[0], t.scale()[1])
                    ));
                }
                Some(Self::get_texture(tex_info, buffers, name, device, queue, tex_layout)?)
            } else { None };
            let normal_tex = if let Some(tex_info) = m.normal_texture() {
                material_buffer_data.normals = 1_u32;
                Some(Self::get_normal_texture(tex_info, buffers, name, device, queue, tex_layout)?)
            } else { None };
            use wgpu::util::DeviceExt;
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&(name.to_string() + " Buffer")),
                contents: bytemuck::cast_slice(&[material_buffer_data]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
            });
            let buffer_bg = BindGroupBuilder::new(name.to_string() + " Bind Group")
                .add_entry(buffer.as_entire_binding())
                .build_bind_group(device, mat_layout);
            
            let material = Material {
                name: name.to_string(),
                buffer,
                buffer_bg,
                diffuse_tex,
                normal_tex,
                ..
            };
            self.materials.push(material);
        }
        Ok(())
    }

    

    pub fn resolve_node(&mut self, n: Node, buffers: &[gltf::buffer::Data], device: &wgpu::Device, transform: cgmath::Matrix4<f32>) {
        let node_transform: cgmath::Matrix4<f32> = n.transform().matrix().into();
        let transform: cgmath::Matrix4<f32> = transform * node_transform;
        let transform_array: [[f32; 4]; 4] = transform.into();
        let ntransform_array: [[f32; 3]; 3] = get_mat3(&transform).invert().unwrap().transpose().into();
        for node in n.children() {
            self.resolve_node(node, buffers, device, transform);
        }
        let name = format!("Mesh {}", n.index());
        let name = n.name().unwrap_or(&name);
        let m = &if let Some(mesh) = n.mesh() { mesh } else { return; };
        let mut prim = 0;
        'prim_iter: for p in m.primitives() {
            prim += 1;
            let name = name.to_string() + &format!("Prim {prim}");
            let mut verts = Vec::new();
            let mut inds = Vec::new();
            let reader = p.reader(|bi| Some(&buffers[bi.index()]));
            let pos = reader.read_positions().unwrap().collect::<Vec<_>>();
            let norm = reader.read_normals().unwrap().collect::<Vec<_>>();
            let uv = reader.read_tex_coords(0).unwrap().into_f32().collect::<Vec<_>>();
            for i in 0..pos.len() {
                verts.push(Vertex {
                    position: pos[i],
                    normal: norm[i],
                    tex_coords: uv[i]
                });
            }
            reader.read_indices().unwrap().into_u32().collect_into(&mut inds);
            let inst_vert = Transform { transform: transform_array, ntransform: ntransform_array };

            if let Some(mat_ind) = p.material().index() {
                'copy_search: for prim_ind in 0..self.materials[mat_ind].mesh_data.len() {
                    let prim = &mut self.materials[mat_ind].mesh_data[prim_ind];
                    if prim.ind_buffer.len() == inds.len() && prim.vert_buffer.len() == verts.len() {
                        for i in 0..(inds.len() - 1) {
                            if inds[i] != prim.ind_buffer[i] { continue 'copy_search; }
                        }
                        for i in 0..(verts.len() - 1) {
                            if verts[i] != prim.vert_buffer[i] { continue 'copy_search; }
                        }
                        prim.inst_buffer.push(inst_vert);
                        continue 'prim_iter;
                    }
                }
                let mesh = Mesh {
                    name,
                    vert_buffer: verts,
                    ind_buffer: inds,
                    inst_buffer: vec![inst_vert]
                };
                self.materials[mat_ind].mesh_data.push(mesh);
            } else {
                'copy_search: for prim_ind in 0..self.mesh_data.len() {
                    let prim = &mut self.mesh_data[prim_ind];
                    if prim.ind_buffer.len() == inds.len() && prim.vert_buffer.len() == verts.len() {
                        for i in 0..(inds.len() - 1) {
                            if inds[i] != prim.ind_buffer[i] { continue 'copy_search; }
                        }
                        for i in 0..(verts.len() - 1) {
                            if verts[i] != prim.vert_buffer[i] { continue 'copy_search; }
                        }
                        prim.inst_buffer.push(inst_vert);
                        continue 'prim_iter;
                    }
                }
                let mesh = Mesh {
                    name,
                    vert_buffer: verts,
                    ind_buffer: inds,
                    inst_buffer: vec![inst_vert]
                };
                self.mesh_data.push(mesh);
            }
        }
    }

    pub fn load(path: &Path, device: &wgpu::Device, queue: &wgpu::Queue,
      tex_layout: &wgpu::BindGroupLayout, mat_layout: &wgpu::BindGroupLayout) -> anyhow::Result<Self> {
        let file = std::fs::File::open(path).map_err(gltf::Error::Io)?;
        let reader = std::io::BufReader::new(file);
        let gltf = gltf::Gltf::from_reader(reader)?;
        let document = gltf.document;
        let buffers = gltf::import_buffers(&document, Some(Path::new("./")), gltf.blob)?;
        let mut out = Self {
            mesh_data: Vec::new(),
            meshes: Vec::new(),
            materials: Vec::new()
        };

        out.load_materials(&document, &buffers, device, queue, tex_layout, mat_layout)?;

        let zero_transform = cgmath::Matrix4::identity();
        for s in document.scenes() {
            for n in s.nodes() {
                out.resolve_node(n, &buffers, device, zero_transform);
            }
        }
        Ok(out)
    }

    pub fn build_meshes(&mut self, device: &wgpu::Device) {
        for mesh_data_ind in 0..self.mesh_data.len() {
            let mesh_data = &self.mesh_data[mesh_data_ind];
            self.meshes.push(mesh_data.build(device));
        }
        for mat_ind in 0..self.materials.len() {
            let mat = &mut self.materials[mat_ind];
            for mesh_data_ind in 0..mat.mesh_data.len() {
                let mesh_data = &mat.mesh_data[mesh_data_ind];
                mat.meshes.push(mesh_data.build(device));
            }
        }
    }
}
