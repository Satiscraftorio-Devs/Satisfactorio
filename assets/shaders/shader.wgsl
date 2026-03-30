// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) uv: i32,
    @location(3) ao: f32
    // @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) ao: f32
    // @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.ao = model.ao;
    // out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0); // 2.
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // return textureSample(t_diffuse, s_diffuse, in.tex_coords);
    // AO: 0 = fully occluded (dark), 3 = fully lit (bright)
    let ambient = 0.25;
    let one_minus_ambient = 1.0 - ambient;
    let ao = clamp(in.ao / 3.0 * one_minus_ambient + ambient, 0.0, 1.0);
    return vec4<f32>(
        in.color[0] * ao,
        in.color[1] * ao,
        in.color[2] * ao,
        1.0
    );
}
