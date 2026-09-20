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

struct InstanceInput {
    @location(5) model0: vec4<f32>,
    @location(6) model1: vec4<f32>,
    @location(7) model2: vec4<f32>,
    @location(8) model3: vec4<f32>,
    @location(9) normal0: vec3<f32>,
    @location(10) normal1: vec3<f32>,
    @location(11) normal2: vec3<f32>,
}

struct CameraUniform {
    world: mat4x4<f32>,
    proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> cam_uni: CameraUniform;

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput
) -> VertexOutput {
    var model_mat: mat4x4<f32> = mat4x4<f32>(
        instance.model0,
        instance.model1,
        instance.model2,
        instance.model3
    );
    var normal_mat: mat3x3<f32> = mat3x3<f32>(
        instance.normal0,
        instance.normal1,
        instance.normal2
    );
    var out: VertexOutput;
    out.vertex_position = model.position;
    out.clip_position = cam_uni.proj * (cam_uni.world * (model_mat * vec4<f32>(model.position, 1.0)));
    out.vertex_normal = normalize(normal_mat * model.normal);
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
}

@group(1) @binding(0)
var<uniform> material: Material;

@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(1)
var s_diffuse: sampler;
@group(3) @binding(0)
var t_normal: texture_2d<f32>;
@group(3) @binding(1)
var s_normal: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex_val: vec4<f32> = textureSample(t_diffuse, s_diffuse, (material.diffuse_transform * vec3<f32>(in.tex_coords, 1.0)).xy);
    var normal: vec3<f32> = in.vertex_normal;
    var material_color: vec4<f32> = vec4<f32>(0.0);
    if material.diffuse.x == -1.0 {
        material_color = tex_val;
    } else {
        material_color = material.diffuse;
    }
    var diffuse: f32 = max(dot(normal, vec3<f32>(0.0, 1.0, 0.0)), 0.0) * material.metal_rough.y + 0.05;
    return vec4<f32>(diffuse) * material_color;
    //return vec4<f32>(normal * vec3<f32>(1.0) + vec3<f32>(0.0), 1.0);
}
