struct CameraUniform {
    view_proj: mat4x4<f32>,
    position: vec3<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
}
@group(1) @binding(0)
var<uniform> light: Light;

struct Phong {
    ka: f32,
    kd: f32,
    ks: f32,
    alph: f32,
}
@group(2) @binding(0)
var<uniform> phong: Phong;


struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) texcoord: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) texcoord: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = vec4<f32>(in.position, 1.0);
    out.world_position = world_pos.xyz;
    out.clip_position = camera.view_proj * world_pos;
    out.world_normal = normalize(in.normal);
    out.texcoord = in.texcoord;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let l = normalize(light.position - in.world_position);
    let diff = max(0.0, dot(l, in.world_normal));
    let v = normalize(camera.position - in.world_position);
    let r = reflect(-l, in.world_normal);
    let spec = pow(max(0.0, dot(v, r)), phong.alph);
    let color = light.color * (phong.ka + phong.kd*diff + phong.ks*spec);
    return vec4<f32>(color, 1.0);
}