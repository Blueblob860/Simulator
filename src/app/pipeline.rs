use wgpu::{BindGroupLayout, ColorTargetState, DepthBiasState, DepthStencilState, Device, Face, FragmentState, FrontFace, MultisampleState, PipelineCompilationOptions, PipelineLayout, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderModule, StencilState, VertexBufferLayout, VertexState};

use crate::app::texture;

pub struct RenderPipelineBuilder<'a> {
    pub name: String,
    pub bind_group_layouts: Vec<Option<&'a BindGroupLayout>> = vec![],
    pub immediate_size: u32 = 0,
    pub vert_shader: Option<(&'a ShaderModule, String)> = None,
    pub frag_shader: Option<(&'a ShaderModule, String)> = None,
    pub vert_buffers: Vec<Option<VertexBufferLayout<'a>>> = vec![],
    pub frag_targets: Vec<Option<ColorTargetState>> = vec![],
    pub front_face: FrontFace = FrontFace::Ccw,
    pub cull_mode: Face = Face::Back,
    pub depth_stencil: Option<DepthStencilState> = None,
    pub multisample: MultisampleState = MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false, }
}

impl<'a> RenderPipelineBuilder<'a> {
    pub fn new(name: String) -> Self {
        Self { name, .. }
    }

    pub fn add_layout(&mut self, layout: Option<&'a BindGroupLayout>) -> &mut Self {
        self.bind_group_layouts.push(layout);
        self
    }

    pub fn set_vert_shader(&mut self, shader: &'a ShaderModule, entry_point: String) -> &mut Self {
        self.vert_shader = Some((shader, entry_point));
        self
    }

    pub fn set_frag_shader(&mut self, shader: &'a ShaderModule, entry_point: String) -> &mut Self {
        self.frag_shader = Some((shader, entry_point));
        self
    }

    pub fn set_shader(&mut self, shader: &'a ShaderModule, vs_entry: String, fs_entry: String) -> &mut Self {
        self.vert_shader = Some((shader, vs_entry));
        self.frag_shader = Some((shader, fs_entry));
        self
    }

    pub fn add_vert_buffer(&mut self, buffer_layout: Option<VertexBufferLayout<'a>>) -> &mut Self {
        self.vert_buffers.push(buffer_layout);
        self
    }

    pub fn add_frag_target(&mut self, target: Option<ColorTargetState>) -> &mut Self {
        self.frag_targets.push(target);
        self
    }

    pub fn set_depth_stencil(&mut self) -> &mut Self {
        self.depth_stencil = Some(wgpu::DepthStencilState {
            format: texture::Texture2d::DEPTH_FORMAT,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::LessEqual),
            stencil: StencilState::default(),
            bias: DepthBiasState::default()
        });
        self
    }

    pub fn build(&mut self, device: &Device) -> (PipelineLayout, RenderPipeline) {
        let ((vert_shader, vs_entry), (frag_shader, fs_entry)) =
            if let Some(vs) = &self.vert_shader
                && let Some(fs) = &self.frag_shader {
                (vs, fs)
            } else {
                panic!();
            };
        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some((self.name.clone() + "_pipeline_layout").as_str()),
            bind_group_layouts: self.bind_group_layouts.as_slice(),
            immediate_size: self.immediate_size
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some((self.name.clone() + "_render_pipeline").as_str()),
            layout: Some(&layout),
            vertex: VertexState {
                module: vert_shader,
                entry_point: Some(vs_entry.as_str()),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &self.vert_buffers
            },
            fragment: Some(FragmentState {
                module: frag_shader,
                entry_point: Some(fs_entry.as_str()),
                compilation_options: PipelineCompilationOptions::default(),
                targets: self.frag_targets.as_slice()
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                polygon_mode: PolygonMode::Fill,
                strip_index_format: None,
                front_face: self.front_face,
                cull_mode: Some(self.cull_mode),
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: self.depth_stencil.clone(),
            multisample: self.multisample,
            multiview_mask: None,
            cache: None,
        });
        (layout, pipeline)
    }
}
