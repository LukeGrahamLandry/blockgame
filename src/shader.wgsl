struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_pos: vec4<f32>
};

struct MeshUniform {
    transform: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> meshInfo: MeshUniform;

struct VertexInput {
    @location(0) world_position: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) uv: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {

    var out: VertexOutput;
    out.clip_position = camera.view_proj * meshInfo.transform * model.world_position;
    out.world_position = meshInfo.transform * model.world_position;
    out.uv = model.uv;
    return out;
}

// these corrispond to texture_bind_group_layout
@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_colour = textureSample(t_diffuse, s_diffuse, in.uv);
    return object_colour;
}
