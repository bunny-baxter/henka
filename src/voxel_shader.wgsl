// Vertex shader

const FACE_COLOR: vec3<f32> = vec3(1.0, 0.81568627, 0.50196078);
const LIGHT_DIRECTION: vec3<f32> = vec3(0.18814417, -0.94072087, 0.28221626);

struct CameraUniform {
    view_projection: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;

@group(1) @binding(0) var texture_sampler: sampler;
@group(1) @binding(1) var texture_view: texture_2d<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) light_mag: f32,
};


@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_projection * vec4<f32>(model.position, 1.0);
    out.uv = model.uv;
    out.light_mag = abs(dot(model.normal, LIGHT_DIRECTION));
    return out;
}


// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_color = textureSample(texture_view, texture_sampler, in.uv);
    let blended_color = mix(FACE_COLOR, texture_color.rgb, 0.35);
    let lit_color = blended_color * in.light_mag;
    return vec4<f32>(lit_color, 1.0);
}
