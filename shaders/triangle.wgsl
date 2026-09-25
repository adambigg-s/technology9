struct VertexIn {
    @location(0) pos: vec3<f32>,
    @location(1) col: vec3<f32>,
};

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) col: vec4<f32>,
};

@group(0) @binding(0) var<uniform> view_proj: mat4x4<f32>;

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;

    out.pos = view_proj * vec4<f32>(in.pos, 1.0);
    out.col = vec4<f32>(in.col, 1.0);

    return out;
}

@fragment

fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return in.col;
}
