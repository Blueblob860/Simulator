use core::f32;
use std::path::Path;

use bytemuck::{Pod, Zeroable};

use crate::app::{buffer::BindGroupBuilder, texture::Texture2d, vertex::Vertex};

pub struct Material {
    pub name: String,
    pub buffer: wgpu::Buffer,
    pub buffer_bg: wgpu::BindGroup,
    pub diffuse_tex: Option<Texture2d> = None,
    pub normal_tex: Option<Texture2d> = None,
    pub emissive_tex: Option<Texture2d> = None,
    pub mr_tex: Option<Texture2d> = None,
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
    pub diffuse_transform: [[f32; 3]; 3],
    pad_4: [u32; 3],
    pub emissive_transform: [[f32; 3]; 3],
    pad_5: [u32; 3],
    pub mr_transform: [[f32; 3]; 3],
    pad_6: [u32; 3],
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
            diffuse_transform: cgmath::Matrix3::identity().into(),
            pad_4: [0_u32; 3],
            emissive_transform: cgmath::Matrix3::identity().into(),
            pad_5: [0_u32; 3],
            mr_transform: cgmath::Matrix3::identity().into(),
            pad_6: [0_u32; 3]
        }
    }
}

pub struct Mesh {
    pub name: String,
    pub vert_buffer: wgpu::Buffer,
    pub ind_buffer: wgpu::Buffer,
    pub transform_buffer: wgpu::Buffer,
    pub transform_bg: wgpu::BindGroup,
    pub items: u32,
    pub material: usize,
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

impl Model {
    pub fn load(path: &Path, device: &wgpu::Device, queue: &wgpu::Queue,
      tex_layout: &wgpu::BindGroupLayout, mat_layout: &wgpu::BindGroupLayout, trans_layout: &wgpu::BindGroupLayout) -> anyhow::Result<Self> {
        let file = std::fs::File::open(path).map_err(gltf::Error::Io)?;
        let reader = std::io::BufReader::new(file);
        let gltf = gltf::Gltf::from_reader(reader)?;
        let document = gltf.document;
        let buffers = gltf::import_buffers(&document, Some(Path::new("./")), gltf.blob)?;
        let mut out = Self {
            meshes: Vec::new(),
            materials: Vec::new()
        };

        for m in document.materials() {
            let name = format!("Material {}", out.materials.len());
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
                    material_buffer_data.diffuse_transform = (
                        cgmath::Matrix3::from_translation(t.offset().into()) *
                        (cgmath::Matrix3::from_angle_z(cgmath::Rad(t.rotation())) *
                        cgmath::Matrix3::from_nonuniform_scale(t.scale()[0], t.scale()[1]))
                    ).into();
                }
                let tex = tex_info.texture();
                let tex_name = name.to_string() + " Diffuse Texture";
                let tex_name = tex.name().unwrap_or(&tex_name);
                let tex_src = tex.source().source();
                match tex_src {
                    gltf::image::Source::View { view, mime_type: _ } => {
                        let mut tex = Texture2d::from_bytes(
                            device,
                            queue, 
                            &buffers[view.buffer().index()][view.offset()..view.offset()+view.length()],
                            tex_name
                        )?;
                        tex.create_bind_group(tex_name.to_string(), device, tex_layout);
                        Some(tex)
                    },
                    gltf::image::Source::Uri { uri, mime_type: _ } => {
                        let mut tex = Texture2d::load_texture(uri, device, queue)?;
                        tex.create_bind_group(tex_name.to_string(), device, tex_layout);
                        Some(tex)
                    }
                }
            } else { None };
            let normal_tex = if let Some(tex_info) = m.normal_texture() {
                material_buffer_data.normals = 1_u32;
                let tex = tex_info.texture();
                let tex_name = name.to_string() + " Normal Texture";
                let tex_name = tex.name().unwrap_or(&tex_name);
                let tex_src = tex.source().source();
                match tex_src {
                    gltf::image::Source::View { view, mime_type: _ } => {
                        let mut tex = Texture2d::from_bytes(
                            device,
                            queue, 
                            &buffers[view.buffer().index()][view.offset()..view.offset()+view.length()],
                            tex_name
                        )?;
                        tex.create_bind_group(tex_name.to_string(), device, tex_layout);
                        Some(tex)
                    },
                    gltf::image::Source::Uri { uri, mime_type: _ } => {
                        let mut tex = Texture2d::load_texture(uri, device, queue)?;
                        tex.create_bind_group(tex_name.to_string(), device, tex_layout);
                        Some(tex)
                    }
                }
            } else { None };
            let emissive_tex = if let Some(tex_info) = m.emissive_texture() {
                material_buffer_data.emissive_color[0] = -1.0;
                if let Some(t) = tex_info.texture_transform() {
                    material_buffer_data.emissive_transform = (
                        cgmath::Matrix3::from_translation(t.offset().into()) *
                        cgmath::Matrix3::from_angle_z(cgmath::Rad(t.rotation())) *
                        cgmath::Matrix3::from_nonuniform_scale(t.scale()[0], t.scale()[1])
                    ).into();
                }
                let tex = tex_info.texture();
                let tex_name = name.to_string() + " Emissive Texture";
                let tex_name = tex.name().unwrap_or(&tex_name);
                let tex_src = tex.source().source();
                match tex_src {
                    gltf::image::Source::View { view, mime_type: _ } => {
                        let mut tex = Texture2d::from_bytes(
                            device,
                            queue, 
                            &buffers[view.buffer().index()][view.offset()..view.offset()+view.length()],
                            tex_name
                        )?;
                        tex.create_bind_group(tex_name.to_string(), device, tex_layout);
                        Some(tex)
                    },
                    gltf::image::Source::Uri { uri, mime_type: _ } => {
                        let mut tex = Texture2d::load_texture(uri, device, queue)?;
                        tex.create_bind_group(tex_name.to_string(), device, tex_layout);
                        Some(tex)
                    }
                }
            } else { None };
            let mr_tex = if let Some(tex_info) = pbr.metallic_roughness_texture() {
                material_buffer_data.mr_base[0] = -1.0;
                if let Some(t) = tex_info.texture_transform() {
                    material_buffer_data.mr_transform = (
                        cgmath::Matrix3::from_translation(t.offset().into()) *
                        cgmath::Matrix3::from_angle_z(cgmath::Rad(t.rotation())) *
                        cgmath::Matrix3::from_nonuniform_scale(t.scale()[0], t.scale()[1])
                    ).into();
                }
                let tex = tex_info.texture();
                let tex_name = name.to_string() + " Metalness Texture";
                let tex_name = tex.name().unwrap_or(&tex_name);
                let tex_src = tex.source().source();
                match tex_src {
                    gltf::image::Source::View { view, mime_type: _ } => {
                        let mut tex = Texture2d::from_bytes(
                            device,
                            queue, 
                            &buffers[view.buffer().index()][view.offset()..view.offset()+view.length()],
                            tex_name
                        )?;
                        tex.create_bind_group(tex_name.to_string(), device, tex_layout);
                        Some(tex)
                    },
                    gltf::image::Source::Uri { uri, mime_type: _ } => {
                        let mut tex = Texture2d::load_texture(uri, device, queue)?;
                        tex.create_bind_group(tex_name.to_string(), device, tex_layout);
                        Some(tex)
                    }
                }
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
                emissive_tex,
                mr_tex,
            };
            out.materials.push(material);
        }

        for n in document.nodes() {
            let name = format!("Mesh {}", n.index());
            let name = n.name().unwrap_or(&name);
            let m = &if let Some(mesh) = n.mesh() { mesh } else { continue; };
            let transform = &n.transform().matrix();
            use wgpu::util::DeviceExt;
            let mut prim = 0;
            for p in m.primitives() {
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

                let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&(name.clone() + "Transform Buffer")),
                    contents: bytemuck::cast_slice(transform),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

                let transform_bg = BindGroupBuilder::new(name.clone() + "Transform")
                    .add_entry(transform_buffer.as_entire_binding())
                    .build_bind_group(device, trans_layout);

                let vert_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&(name.clone() + "Vertex Buffer")),
                    contents: bytemuck::cast_slice(verts.as_slice()),
                    usage: wgpu::BufferUsages::VERTEX,
                });

                let ind_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&(name.clone() + "Index Buffer")),
                    contents: bytemuck::cast_slice(inds.as_slice()),
                    usage: wgpu::BufferUsages::INDEX,
                });

                out.meshes.push(Mesh {
                    name: name,
                    vert_buffer, ind_buffer,
                    transform_buffer, transform_bg,
                    items: inds.len() as u32,
                    material: p.material().index().unwrap_or(0)
                });
            }
        }
        Ok(out)
    }
}