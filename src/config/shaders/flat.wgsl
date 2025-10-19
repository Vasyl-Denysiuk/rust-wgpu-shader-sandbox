struct CameraUniform {
    proj: mat4x4<f32>,
    view: mat4x4<f32>,
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

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) @interpolate(flat) color: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_pos = vec4<f32>(in.position, 1.0);
    let world_normal = normalize((camera.view * vec4<f32>(in.normal, 0.0)).xyz);

    let light_pos = (camera.view * vec4<f32>(light.position, 1)).xyz;
    let l = normalize(light_pos - world_pos.xyz);
    let v = normalize(-world_pos.xyz);
    let r = reflect(-l, world_normal);
    let diff = max(0.0, dot(l, world_normal));
    let spec = pow(max(0.0, dot(v, r)), phong.alph);

    let color = light.color * (phong.ka + phong.kd * diff + phong.ks * spec);

    out.clip_position = camera.proj * camera.view * world_pos;
    out.color = color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}