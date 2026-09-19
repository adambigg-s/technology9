struct VertexIn {
    @location(0) pos: vec3<f32>,
    @location(1) nor: vec3<f32>,
    @location(2) tex: vec2<f32>,
};

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) world_pos: vec4<f32>,
    @location(1) nor: vec3<f32>,
    @location(2) tex: vec2<f32>,
};

@group(0) @binding(0) var diffuse_atlas: texture_2d<f32>;
@group(0) @binding(1) var normal_atlas: texture_2d<f32>;
@group(0) @binding(2) var specular_atlas: texture_2d<f32>;
@group(0) @binding(3) var sample_atlas: sampler;
@group(0) @binding(4) var<uniform> view_proj: mat4x4<f32>;
@group(0) @binding(5) var<uniform> view: mat4x4<f32>;
@group(0) @binding(6) var<uniform> screen_ar: f32;
@group(0) @binding(7) var<uniform> flashlight: f32;
@group(0) @binding(8) var<uniform> time: f32;

@group(1) @binding(0) var<uniform> model: mat4x4<f32>;

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;

    out.pos = view_proj * model * vec4<f32>(in.pos, 1.0);
    out.world_pos = (view * model * vec4<f32>(in.pos, 1.0));
    out.nor = (view * vec4<f32>(in.nor, 1.0)).xyz;
    out.tex = in.tex;

    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let color = textureSample(diffuse_atlas, sample_atlas, in.tex);

    let dist = length(in.world_pos.xyz / in.world_pos.w);
    let proxy = 1.0 - smoothstep(4.0, 16.0, dist);

    let quantized_speed = floor(proxy * 4.0) / 4.0;
    let current_speed = mix(16.0, 32.0, proxy);
    let flicker_wave = cos(time * current_speed);

    let lum = mix(1.0, flicker_wave, proxy);
    let attenuated = smoothstep(32.0, 4.0, dist);

    let final_color = color * lum * attenuated;

    return final_color;
}

