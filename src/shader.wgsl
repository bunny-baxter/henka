// Vertex shader

const FACE_COLOR: vec3<f32> = vec3(1.0, 0.81568627, 0.50196078);
const LIGHT_DIRECTION: vec3<f32> = vec3(0.18814417, -0.94072087, 0.28221626);

struct CameraUniform {
    view_projection: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};


@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_projection * vec4<f32>(model.position, 1.0);
    var light_mag = abs(dot(model.normal, LIGHT_DIRECTION));
    out.color = FACE_COLOR * light_mag;
    return out;
}


// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color.r, in.color.g, in.color.b, 1.0);
}
