// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: u32,
    @location(2) tex_layer: f32,
    @location(3) ao: f32,
    @location(4) u: f32,
    @location(5) v: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) ao: f32,
    @location(2) tex_layer: f32,
    @location(3) uv: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.color = vec4<f32>(
        f32(model.color >> 24) / 255.0,
        f32((model.color >> 16) & 0xFFu) / 255.0,
        f32((model.color >> 8) & 0xFFu) / 255.0,
        f32(model.color & 0xFFu) / 255.0,
    );
    out.ao = model.ao;
    out.tex_layer = model.tex_layer;
    out.uv = vec2<f32>(model.u, model.v);
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d_array<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex_color: vec4<f32>;
    
    if (in.tex_layer != 4294967295) {
        tex_color = textureSample(
            t_diffuse,
            s_diffuse,
            in.uv,
            u32(in.tex_layer)
        );
    }
    else {
        tex_color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    }

    // AO: 0 = fully occluded (dark), 3 = fully lit (bright)
    let ambient = 0.25;
    let one_minus_ambient = 1.0 - ambient;
    let ao = clamp(in.ao / 3.0 * one_minus_ambient + ambient, 0.0, 1.0);
    return vec4<f32>(
        tex_color.r * in.color[0] * ao,
        tex_color.g * in.color[1] * ao,
        tex_color.b * in.color[2] * ao,
        tex_color.a * in.color[3],
    );
}
