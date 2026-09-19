const PI = cos(-1.0);

const BACKROOMS_EPS: f32 = 1e-3;

const BACKROOMS_AMBIENT: f32 = 0.01;
const BACKROOMS_LIGHT: vec4<f32> = vec4<f32>(1.0, 0.90, 0.75, 1.0);

const FADE_COLOR: vec3<f32> = vec3<f32>(0.005, 0.0, 0.0);

const FOG_EXP: f32 = 1.5;
const FOG_START: f32 = 25.0;
const FOG_END: f32 = 100.0;

const FL_END: f32 = 150.0;
const FL_INNER: f32 = 0.2;
const FL_OUTER: f32 = 1.5;
const FL_STRENGTH: f32 = 5.0;

struct VertexIn {
    @location(0) pos: vec3<f32>,
    @location(1) nor: vec3<f32>,
    @location(2) tex: vec2<f32>,
    @location(3) fil: f32,
    @location(4) bil: f32,
    @location(5) ao: f32,
};

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) world_pos: vec4<f32>,
    @location(1) nor: vec3<f32>,
    @location(2) tex: vec2<f32>,
    @location(3) fil: f32,
    @location(4) ao: f32,
    @location(5) @interpolate(linear) ndc: vec3<f32>,
}

@group(0) @binding(0) var diffuse_atlas: texture_2d<f32>;
@group(0) @binding(1) var normal_atlas: texture_2d<f32>;
@group(0) @binding(2) var specular_atlas: texture_2d<f32>;
@group(0) @binding(3) var sample_atlas: sampler;
@group(0) @binding(4) var<uniform> view_proj: mat4x4<f32>;
@group(0) @binding(5) var<uniform> view: mat4x4<f32>;
@group(0) @binding(6) var<uniform> screen_ar: f32;
@group(0) @binding(7) var<uniform> flashlight: f32;
@group(0) @binding(8) var<uniform> time: f32;

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;

    out.pos = view_proj * vec4<f32>(in.pos, 1.0);
    out.world_pos = view * vec4<f32>(in.pos, 1.0);
    out.nor = (view * vec4<f32>(in.nor, 0.0)).xyz;
    out.tex = in.tex;
    out.fil = in.fil;
    out.ao = in.ao;
    out.ndc = out.pos.xyz / out.pos.w;

    return out;
}

struct FragmentOutput {
    @builtin(frag_depth) depth: f32,
    @location(0) color: vec4<f32>,
};

@fragment
fn fs_main(in: VertexOut) -> FragmentOutput {
    var out: FragmentOutput;

    let diffuse_color = textureSampleBias(diffuse_atlas, sample_atlas, in.tex, -0.25);
    let normal_color = textureSample(normal_atlas, sample_atlas, in.tex).xyz * 2.0 - 1.0;
    let specular_color = textureSample(specular_atlas, sample_atlas, in.tex);

    if diffuse_color.a < BACKROOMS_EPS {
        discard;
    }

    let dp1 = dpdx(in.world_pos.xyz);
    let dp2 = dpdy(in.world_pos.xyz);
    let duv1 = dpdx(in.tex);
    let duv2 = dpdy(in.tex);
    let det = duv1.x * duv2.y - duv2.x * duv1.y;
    let inv_det = 1.0 / det;
    let tangent = normalize((dp1 * duv2.y - dp2 * duv1.y) * inv_det);
    let bitangent = normalize((-dp1 * duv2.x + dp2 * duv1.x) * inv_det);
    let face_normal = normalize(in.nor);
    let tbn = mat3x3<f32>(tangent, bitangent, face_normal);
    let normal_map_strength = 0.25;
    let normal = normalize(mix(face_normal, tbn * normal_color, normal_map_strength));

    let dist = length(in.world_pos.xyz);
    let fog_factor = pow(clamp((dist - FOG_START) / (FOG_END - FOG_START), 0.0, 1.0), FOG_EXP);

    let cone_center = vec2<f32>(in.ndc.x * screen_ar, in.ndc.y);
    let cone_dist = length(cone_center);
    let spot_factor = pow(smoothstep(FL_OUTER, FL_INNER, cone_dist), 3.0);
    let fl_attenuation = distance_attenuation(dist, FL_END);

    let view_dir = normalize(-in.world_pos.xyz);
    let light_dir = view_dir;
    let fl_intensity = max(dot(normal, light_dir), 0.0);
    let fl_diffuse = spot_factor * fl_attenuation * fl_intensity * FL_STRENGTH * flashlight;
    let specular_shape = specular(normal, view_dir, light_dir, 32.0);
    let specular_intensity = 0.75;
    let fl_specular = specular_shape * specular_color.rgb * specular_intensity * spot_factor * fl_attenuation * fl_intensity * flashlight;

    let face_light = dot(face_normal, normal) * in.fil;
    let light_boost = 1.5;
    let ambient_lum = pow(clamp(face_light, BACKROOMS_AMBIENT, 1.0), 2.0) * light_boost;
    let tint_mix = smoothstep(0.8, 1.0, face_light);
    let light_color = mix(BACKROOMS_LIGHT, vec4<f32>(1.0), tint_mix);

    let ambient_light_dir = face_normal;
    let ambient_intensity = max(dot(normal, ambient_light_dir), 0.0);
    let ambient_shine = 16.0;
    let ambient_specular_shape = specular(normal, view_dir, ambient_light_dir, ambient_shine);
    let ambient_specular_intensity = 0.25;
    let ambient_specular = ambient_specular_shape * ambient_specular_intensity * specular_color.rgb * ambient_intensity * in.fil;

    let total_light = (vec3<f32>(ambient_lum + fl_diffuse) + fl_specular + ambient_specular) * light_color.rgb;
    let shaded_color = vec4<f32>(diffuse_color.rgb * in.ao * total_light, diffuse_color.a);

    let final_color = mix(shaded_color, vec4<f32>(FADE_COLOR, 1.0), fog_factor);

    out.depth = in.pos.z;
    out.color = clamp(final_color, vec4<f32>(0.0), vec4<f32>(1.0));

    return out;
}

fn distance_attenuation(dist: f32, range: f32) -> f32 {
    let window = clamp(1.0 - pow(dist / range, 4.0), 0.0, 1.0);
    return window * window / max(dist * dist, 1.0);
}

fn specular(normal: vec3<f32>, view_dir: vec3<f32>, light_dir: vec3<f32>, shine: f32) -> f32 {
    let half_dir = normalize(view_dir + light_dir);
    let intensity = max(dot(normal, half_dir), 0.0);
    let norm_factor = (shine + 8.0) / (8.0 * PI);
    return norm_factor * pow(intensity, shine);
}
