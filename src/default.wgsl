struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) texcoord: vec2<f32>,
};
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) texcoord: vec2<f32>,
};
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
	let light_position = vec3f(3.0, 3.0, 3.0);
	let light = normalize(light_position - in.position);
	let coef = dot(light, normalize(in.normal));
	let color = vec3f(1.0, 1.0, 0.0)*(0.05+ clamp(coef, 0, 1));
	out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.normal = color;
    out.texcoord = in.texcoord;
    return out;
}
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4f(in.normal, 1.0);
}