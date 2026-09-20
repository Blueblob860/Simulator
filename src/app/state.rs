use std::{path::Path, sync::Arc};

use wgpu::util::DeviceExt;
use winit::{event::{ElementState, MouseButton}, event_loop::ActiveEventLoop, keyboard::KeyCode, window::Window};

use crate::app::{buffer::BindGroupBuilder, camera::Camera, egui::EguiImpl, gui::ViewportUi, model::Model, vertex::{Transform, Vertex}};

pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    pub window: Arc<Window>,
    pub egui_state: EguiImpl,
    gui_state: ViewportUi,
    render_pipeline: wgpu::RenderPipeline,
    depth_texture: crate::app::texture::Texture2d,
    model: Model,
    tex_bind_group: wgpu::BindGroup,
    camera: Camera,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    keys_pressed: std::collections::HashMap<KeyCode, bool>,
    mouse_buttons_pressed: std::collections::HashMap<MouseButton, bool>,
    mouse_delta: cgmath::Vector2<f32>,
}

impl State {
    pub async fn new(window: Arc<Window>, v5_disp_output: crate::FrameSubType, v5_input: crate::app::input::V5InputHandler) -> anyhow::Result<Self> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
            apply_limit_buckets: true,
        }).await?;

        if !adapter.get_downlevel_capabilities().flags.contains(wgpu::DownlevelFlags::INDIRECT_EXECUTION) { panic!("Indirect execution not supported!"); }

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::MULTI_DRAW_INDIRECT_COUNT,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            required_limits: wgpu::Limits { max_bind_groups: 8, ..Default::default() },
            memory_hints: Default::default(),
            trace: wgpu::Trace::Off
        }).await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb()).copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };

        let egui_state = EguiImpl::new(&device, config.format, &window);
        let gui_state = ViewportUi::new(&egui_state.context, v5_input, v5_disp_output);

        let diffuse_bytes = include_bytes!("../../assets/happy-tree.png");
        let diffuse_texture = crate::app::texture::Texture2d::from_bytes(&device, &queue, diffuse_bytes, "diffuse_texture", (wgpu::AddressMode::ClampToEdge, wgpu::AddressMode::ClampToEdge)).unwrap();

        let (texture_bind_group_layout, tex_bind_group) = crate::app::buffer::BindGroupBuilder::new("diffuse_texture".to_string())
            .add_layout_entry(wgpu::ShaderStages::FRAGMENT, wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true }
            }).add_layout_entry(wgpu::ShaderStages::FRAGMENT, 
                wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
            ).add_entry(wgpu::BindingResource::TextureView(&diffuse_texture.view))
            .add_entry(wgpu::BindingResource::Sampler(&diffuse_texture.sampler))
            .build(&device);

        let camera = Camera::new(
            (0., 0.5, -1.).into(),
            (0.0, 0.0, 0.0).into(),
            config.width, config.height, 
            90.0, 0.01, 100.0
        );
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera.uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        });

        let (camera_bind_group_layout, camera_bind_group) = crate::app::buffer::BindGroupBuilder::new("camera".to_string())
            .add_layout_entry(wgpu::ShaderStages::VERTEX, wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None
            }).add_entry(camera_buffer.as_entire_binding())
            .build(&device);

        let mat_bg_layout = BindGroupBuilder::new("Material Entry".to_string())
            .add_layout_entry(wgpu::ShaderStages::FRAGMENT, wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None
            }).build_layout(&device);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../assets/shader.wgsl").into()),
        });

        let depth_texture = crate::app::texture::Texture2d::create_depth_tex(&device, &config, "main_depth_buffer_texture");
        let (_render_pipeline_layout, render_pipeline) =
            crate::app::pipeline::RenderPipelineBuilder::new("main".to_string())
                .add_layout(Some(&camera_bind_group_layout)) // 0 Camera
                .add_layout(Some(&mat_bg_layout)) // 1 Material Buffer
                .add_layout(Some(&texture_bind_group_layout)) // 2 Diffuse Tex
                .add_layout(Some(&texture_bind_group_layout)) // 3 Normal Tex
                .set_shader(&shader, "vs_main".to_string(), "fs_main".to_string())
                .add_vert_buffer(Some(Vertex::desc()))
                .add_vert_buffer(Some(Transform::desc()))
                .add_frag_target(Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL
                })).set_depth_stencil().build(&device);

        let mut model = Model::load(
            Path::new("test_dt_normal.glb"),
            &device, &queue,
            &texture_bind_group_layout,
            &mat_bg_layout
        ).unwrap();
        model.build_meshes(&device);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            window,
            egui_state,
            gui_state,
            render_pipeline,
            depth_texture,
            model,
            tex_bind_group,
            camera,
            camera_buffer,
            camera_bind_group,
            keys_pressed: std::collections::HashMap::new(),
            mouse_buttons_pressed: std::collections::HashMap::new(),
            mouse_delta: cgmath::vec2(0.0, 0.0),
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width; self.config.height = height;
            self.depth_texture = crate::app::texture::Texture2d::create_depth_tex(&self.device, &self.config, "main_depth_buffer_texture");
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();

        if !self.is_surface_configured { return Ok(()); }

        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_tex) => surface_tex,
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_tex) => surface_tex,
            wgpu::CurrentSurfaceTexture::Timeout
             | wgpu::CurrentSurfaceTexture::Occluded
             | wgpu::CurrentSurfaceTexture::Validation => {
                return Ok(());
            },
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            },
            wgpu::CurrentSurfaceTexture::Lost => {
                anyhow::bail!("Lost device!");
            }
        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0, g: 0.4, b: 1.0, a: 1.0
                        }),
                        store: wgpu::StoreOp::Store,
                    }
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store
                    }),
                    stencil_ops: None
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None
            });

            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(2, &self.tex_bind_group, &[]);
            render_pass.set_bind_group(3, &self.tex_bind_group, &[]);

            for mesh in &self.model.meshes {
                render_pass.set_vertex_buffer(0, mesh.vert_buffer.slice(..));
                render_pass.set_vertex_buffer(1, mesh.inst_buffer.slice(..));
                render_pass.set_index_buffer(mesh.ind_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..mesh.items, 0, 0..mesh.instances as _);
            }

            for mat in &self.model.materials {
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(1, &mat.buffer_bg, &[]);
                if let Some(diffuse) = &mat.diffuse_tex
                    { render_pass.set_bind_group(2, diffuse.bind_group.as_ref().unwrap(), &[]); }
                if let Some(normal) = &mat.normal_tex
                    { render_pass.set_bind_group(3, normal.bind_group.as_ref().unwrap(), &[]); }
                for mesh in &mat.meshes {
                    render_pass.set_vertex_buffer(0, mesh.vert_buffer.slice(..));
                    render_pass.set_vertex_buffer(1, mesh.inst_buffer.slice(..));
                    render_pass.set_index_buffer(mesh.ind_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..mesh.items, 0, 0..mesh.instances as _);
                }
            }
        }

        {
            let egui_raw_input = self.egui_state.state.take_egui_input(&self.window);
            let full_output = self.egui_state.state.egui_ctx()
                .run_ui(egui_raw_input, |ui| self.gui_state.ui(ui));

            let screen_descriptor = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [self.config.width, self.config.height],
                pixels_per_point: self.window.current_monitor().map(|v| v.scale_factor()).unwrap_or(1.0) as f32
            };
            self.egui_state.state.egui_ctx().set_pixels_per_point(screen_descriptor.pixels_per_point);
            self.egui_state.state.handle_platform_output(&self.window, full_output.platform_output);
            let tris = self.egui_state.state.egui_ctx()
                .tessellate(full_output.shapes, self.egui_state.state.egui_ctx().pixels_per_point());
            for (id, img_delta) in &full_output.textures_delta.set {
                self.egui_state.renderer.update_texture(&self.device, &self.queue, *id, &img_delta[0]);
            }
            self.egui_state.renderer.update_buffers(&self.device, &self.queue, &mut encoder, &tris, &screen_descriptor);
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui_root_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store
                    }
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None
            });
            self.egui_state.renderer.render(&mut render_pass.forget_lifetime(), &tris, &screen_descriptor);
            for x in &full_output.textures_delta.free {
                self.egui_state.renderer.free_texture(x);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(output);
        Ok(())
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        self.keys_pressed.insert(code, is_pressed);
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }

    pub fn handle_mouse_movement(&mut self, dx: f64, dy: f64) {
        self.mouse_delta = cgmath::vec2(dx as f32, dy as f32);
    }

    pub fn handle_mouse_input(&mut self, button: MouseButton, state: ElementState) {
        self.mouse_buttons_pressed.insert(button, state.is_pressed());
    }

    pub fn key_pressed(&self, code: KeyCode) -> bool {
        *self.keys_pressed.get(&code).unwrap_or(&false)
    }

    pub fn update(&mut self) {
        self.camera.update((
                self.key_pressed(KeyCode::KeyW),
                self.key_pressed(KeyCode::KeyA),
                self.key_pressed(KeyCode::KeyS),
                self.key_pressed(KeyCode::KeyD),
                self.key_pressed(KeyCode::Space),
                self.key_pressed(KeyCode::ShiftLeft),
            ), self.mouse_delta / (self.config.width as f32).min(self.config.height as f32),
            *self.mouse_buttons_pressed.get(&MouseButton::Left).unwrap_or(&false)
        );
        self.mouse_delta = cgmath::vec2(0.0, 0.0);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera.uniform]));
    }
}
