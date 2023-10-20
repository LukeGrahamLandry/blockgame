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
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {

    var out: VertexOutput;
    out.clip_position = camera.view_proj * meshInfo.transform * model.world_position;
    out.world_position = meshInfo.transform * model.world_position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let r = 1.0 - f32(u32(in.world_position.x) % u32(16)) / 16.0;
    let g = f32(u32(in.world_position.y) % u32(16)) / 16.0;
    let b = f32(u32(in.world_position.z) % u32(16)) / 16.0;
    return vec4(r, g, b, 1.0);
}
