// Shared

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vertex_position: vec3<f32>,
    @location(1) vertex_normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
};

// Vertex Shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
}

struct CameraUniform {
    world: mat4x4<f32>,
    proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> cam_uni: CameraUniform;
@group(1) @binding(0)
var<uniform> transform: mat4x4<f32>;

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.vertex_position = model.position;
    out.clip_position = cam_uni.proj * (cam_uni.world * (transform * vec4<f32>(model.position, 1.0)));
    out.vertex_normal = model.normal;
    out.tex_coords = model.tex_coords;
    return out;
}

// Fragment/Pixel Shader

struct Material {
    diffuse: vec4<f32>,
    normals: u32,
    emissive: vec3<f32>,
    metal_rough: vec2<f32>,
    diffuse_transform: mat3x3<f32>,
    emissive_transform: mat3x3<f32>,
    mr_transform: mat3x3<f32>,
}

@group(2) @binding(0)
var<uniform> material: Material;

@group(3) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(3) @binding(1)
var s_diffuse: sampler;
@group(4) @binding(0)
var t_normal: texture_2d<f32>;
@group(4) @binding(1)
var s_normal: sampler;
@group(5) @binding(0)
var t_emissive: texture_2d<f32>;
@group(5) @binding(1)
var s_emissive: sampler;
@group(6) @binding(0)
var t_metal: texture_2d<f32>;
@group(6) @binding(1)
var s_metal: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex_val: vec4<f32> = textureSample(t_diffuse, s_diffuse, (material.diffuse_transform * vec3<f32>(in.tex_coords, 1.0)).xy);
    // return vec4<f32>(in.vertex_normal / 2.0 + vec3<f32>(0.5), 1.0);
    if material.diffuse.x == -1.0 {
        return tex_val;
    } else {
        return material.diffuse;
    }
}