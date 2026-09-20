use image::GenericImageView;
use wgpu::AddressMode;

use crate::app::{buffer::BindGroupBuilder, resources::load_bytes};

pub struct Texture2d {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl Texture2d {
    pub fn load_texture(file: &str, device: &wgpu::Device, queue: &wgpu::Queue, wrap: (AddressMode, AddressMode)) -> anyhow::Result<Self> {
        let data = load_bytes(file)?;
        Self::from_bytes(device, queue, data.as_slice(), file, wrap)
    }

    pub fn from_bytes(device: &wgpu::Device, queue: &wgpu::Queue, bytes: &[u8], label: &str, wrap: (AddressMode, AddressMode)) -> anyhow::Result<Self> {
        Self::from_image(device, queue, &image::load_from_memory(bytes)?, label, wrap)
    }

    pub fn from_image(device: &wgpu::Device, queue: &wgpu::Queue, image: &image::DynamicImage, label: &str, wrap: (AddressMode, AddressMode)) -> anyhow::Result<Self> {
        let rgba = image.to_rgba8();
        let dimensions = image.dimensions();

        let tex_size = wgpu::Extent3d {
            width: dimensions.0, height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: tex_size,
            mip_level_count: 1,
            sample_count: 1, 
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some(label),
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            tex_size
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wrap.0,
            address_mode_v: wrap.1,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self { texture, view, sampler, bind_group_layout: None, bind_group: None })
    }

    pub fn create_bind_group_layout(&mut self, label: String, device: &wgpu::Device) {
        self.bind_group_layout = Some(BindGroupBuilder::new(label)
            .add_layout_entry(wgpu::ShaderStages::FRAGMENT, wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true }
            }).add_layout_entry(wgpu::ShaderStages::FRAGMENT, 
                wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
            ).build_layout(device));
    }

    pub fn create_bind_group(&mut self, label: String, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) {
        self.bind_group = Some(BindGroupBuilder::new(label)
            .add_entry(wgpu::BindingResource::TextureView(&self.view))
            .add_entry(wgpu::BindingResource::Sampler(&self.sampler))
            .build_bind_group(device, layout));
    }

    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn create_depth_tex(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, label: &str) -> Self {
        let size = wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1, 
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            label: Some(label),
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self { texture, view, sampler, bind_group_layout: None, bind_group: None }
    }
}