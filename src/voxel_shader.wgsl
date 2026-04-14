const AMBIENT: f32 = 0.04;
const FACE_COLOR: vec3<f32> = vec3(1.0, 0.81568627, 0.50196078);

struct CameraUniform {
    view_projection: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct LightingUniform {
  sun_position: vec3<f32>,
};
@group(1) @binding(0) var<uniform> lighting: LightingUniform;

@group(2) @binding(0) var texture_sampler: sampler;
@group(2) @binding(1) var texture_view: texture_2d<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) light: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) light: vec3<f32>,
    @location(2) lambertian_factor: f32,
};


// Vertex shader

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_projection * vec4<f32>(model.position, 1.0);
    out.uv = model.uv;
    out.light = model.light;
    out.lambertian_factor = max(dot(model.normal, -lighting.sun_position), 0.0);
    return out;
}


// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_color = textureSample(texture_view, texture_sampler, in.uv);
    let face_base_color = mix(FACE_COLOR, texture_color.rgb, 0.35);
    let lit_color = AMBIENT + face_base_color * in.light * in.lambertian_factor;
    return vec4<f32>(lit_color, 1.0);
}
