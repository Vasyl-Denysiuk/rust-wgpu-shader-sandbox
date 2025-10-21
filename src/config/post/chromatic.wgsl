const red_offset = vec2<f32>(0.005, 0.0);
const blue_offset = vec2<f32>(-0.005, 0.0);
const green_offset = vec2<f32>(0.005, 0.0);

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>
) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(position, 0.0, 1.0);
    out.uv = uv;
    return out;
}

@group(0) @binding(0) var post_texture: texture_2d<f32>;
@group(0) @binding(1) var post_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let test = vec2<f32>(1.0, 1.0);
    let red = textureSample(post_texture, post_sampler, in.uv + red_offset).r;
    let green = textureSample(post_texture, post_sampler, in.uv + green_offset).g;
    let blue_a = textureSample(post_texture, post_sampler, in.uv + blue_offset).ba;
    return vec4<f32>(red, green, blue_a);
}