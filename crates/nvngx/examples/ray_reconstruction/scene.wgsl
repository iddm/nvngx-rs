// Procedural ray-traced scene used by the DLSS-RR demo.
//
// Each invocation traces one primary ray against an analytic scene
// (ground plane + three spheres), takes a single Monte Carlo sample of
// the diffuse BRDF for direct lighting, and writes one G-buffer entry
// for DLSS-RR to denoise.
//
// Output bindings follow the order the host expects:
//   0: noisy color (rgba16f)
//   1: depth (r32f, linearised, normalised to [0, 1])
//   2: world-space normals encoded as (n*0.5 + 0.5) (rgba16f)
//   3: roughness (r8unorm)
//   4: diffuse albedo (rgba8unorm)
//   5: specular albedo (rgba8unorm)
//   6: motion vectors in pixel space (rg16f)
//
// Push constants drive the camera + per-frame jitter and supply a
// frame index used to seed the per-pixel RNG so the noise is
// temporally uncorrelated.

@group(0) @binding(0) var color_out: texture_storage_2d<rgba16float, write>;
@group(0) @binding(1) var depth_out: texture_storage_2d<r32float, write>;
@group(0) @binding(2) var normals_out: texture_storage_2d<rgba16float, write>;
@group(0) @binding(3) var roughness_out: texture_storage_2d<r8unorm, write>;
@group(0) @binding(4) var diffuse_albedo_out: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(5) var specular_albedo_out: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(6) var mv_out: texture_storage_2d<rg16float, write>;

struct Params {
    canvas_size: vec2<u32>,
    jitter: vec2<f32>,
    frame_index: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
}

@group(0) @binding(7) var<uniform> pc: Params;

const PI: f32 = 3.14159265358979;

// PCG-style 32-bit hash, then unpack to two floats in [0, 1).
fn hash_u32(seed: u32) -> u32 {
    var v = seed * 747796405u + 2891336453u;
    v = ((v >> ((v >> 28u) + 4u)) ^ v) * 277803737u;
    return (v >> 22u) ^ v;
}

fn rand2(coord: vec2<u32>, frame: u32) -> vec2<f32> {
    let s0 = hash_u32(coord.x * 1664525u + coord.y * 1013904223u + frame * 0xdeadbeefu);
    let s1 = hash_u32(s0 ^ 0xcafebabeu);
    return vec2<f32>(f32(s0) / 4294967296.0, f32(s1) / 4294967296.0);
}

// Cosine-weighted hemisphere sample around `n`.
fn cosine_hemisphere(n: vec3<f32>, rng: vec2<f32>) -> vec3<f32> {
    let r = sqrt(rng.x);
    let phi = 2.0 * PI * rng.y;
    let local = vec3<f32>(r * cos(phi), r * sin(phi), sqrt(max(0.0, 1.0 - rng.x)));
    // Build TBN around `n`.
    let up = select(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), abs(n.y) < 0.999);
    let t = normalize(cross(up, n));
    let b = cross(n, t);
    return normalize(local.x * t + local.y * b + local.z * n);
}

struct Hit {
    t: f32,
    pos: vec3<f32>,
    normal: vec3<f32>,
    albedo: vec3<f32>,
    roughness: f32,
}

fn no_hit() -> Hit {
    return Hit(-1.0, vec3<f32>(0.0), vec3<f32>(0.0), vec3<f32>(0.0), 1.0);
}

// Ray vs. axis-aligned plane y=0 (floor).
fn intersect_plane(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    if (abs(rd.y) < 1e-4) { return -1.0; }
    let t = -ro.y / rd.y;
    return select(-1.0, t, t > 0.001);
}

// Ray vs. sphere. Returns nearest positive root or -1.
fn intersect_sphere(ro: vec3<f32>, rd: vec3<f32>, c: vec3<f32>, r: f32) -> f32 {
    let oc = ro - c;
    let b = dot(oc, rd);
    let qc = dot(oc, oc) - r * r;
    let h = b * b - qc;
    if (h < 0.0) { return -1.0; }
    let s = sqrt(h);
    let t0 = -b - s;
    let t1 = -b + s;
    if (t0 > 0.001) { return t0; }
    if (t1 > 0.001) { return t1; }
    return -1.0;
}

fn trace_scene(ro: vec3<f32>, rd: vec3<f32>) -> Hit {
    var h = no_hit();
    var best_t: f32 = 1e30;

    // Floor.
    let tp = intersect_plane(ro, rd);
    if (tp > 0.0 && tp < best_t) {
        best_t = tp;
        let p = ro + rd * tp;
        // Checker pattern albedo.
        let cx = i32(floor(p.x * 0.5));
        let cz = i32(floor(p.z * 0.5));
        let checker = ((cx + cz) & 1) == 0;
        let alb = select(vec3<f32>(0.18), vec3<f32>(0.55), checker);
        h = Hit(tp, p, vec3<f32>(0.0, 1.0, 0.0), alb, 1.0);
    }

    // Three spheres on the plane.
    let spheres = array<vec4<f32>, 3>(
        vec4<f32>(-1.5, 1.0, 0.0, 1.0),
        vec4<f32>(0.0,  1.0, 0.0, 1.0),
        vec4<f32>(1.5,  1.0, 0.0, 1.0),
    );
    let albedos = array<vec3<f32>, 3>(
        vec3<f32>(0.85, 0.20, 0.20),
        vec3<f32>(0.20, 0.85, 0.30),
        vec3<f32>(0.20, 0.40, 0.90),
    );
    let roughs = array<f32, 3>(0.25, 0.55, 0.85);

    for (var i = 0u; i < 3u; i++) {
        let s = spheres[i];
        let ts = intersect_sphere(ro, rd, s.xyz, s.w);
        if (ts > 0.0 && ts < best_t) {
            best_t = ts;
            let p = ro + rd * ts;
            let n = normalize(p - s.xyz);
            h = Hit(ts, p, n, albedos[i], roughs[i]);
        }
    }

    return h;
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let size = vec2<i32>(i32(pc.canvas_size.x), i32(pc.canvas_size.y));
    let coord = vec2<i32>(i32(gid.x), i32(gid.y));
    if (coord.x >= size.x || coord.y >= size.y) { return; }

    // Static camera looking at the spheres from a slight elevation.
    let cam_pos = vec3<f32>(0.0, 1.5, 5.0);
    let cam_target = vec3<f32>(0.0, 1.0, 0.0);
    let cam_up = vec3<f32>(0.0, 1.0, 0.0);
    let cam_fwd = normalize(cam_target - cam_pos);
    let cam_right = normalize(cross(cam_fwd, cam_up));
    let cam_v = cross(cam_right, cam_fwd);

    let aspect = f32(size.x) / f32(size.y);
    let fov_tan = tan(0.5 * 0.9); // ~52° vertical fov
    let uv = (vec2<f32>(coord) + vec2<f32>(0.5) + pc.jitter) / vec2<f32>(size);
    let ndc = vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
    let rd = normalize(
        cam_fwd
        + cam_right * (ndc.x * aspect * fov_tan)
        + cam_v * (ndc.y * fov_tan)
    );
    let ro = cam_pos;

    let h = trace_scene(ro, rd);

    var color: vec3<f32>;
    var depth: f32 = 1.0;
    var normal: vec3<f32> = vec3<f32>(0.0, 1.0, 0.0);
    var albedo: vec3<f32> = vec3<f32>(0.05);
    var roughness: f32 = 1.0;

    if (h.t > 0.0) {
        // 1-spp diffuse Monte Carlo. Sky is the only "light" — no
        // direct light sources, so each frame we sample a single
        // hemispherical direction and shade against the sky dome.
        let rng = rand2(vec2<u32>(u32(coord.x), u32(coord.y)), pc.frame_index);
        let bounce = cosine_hemisphere(h.normal, rng);
        // Sun direction — strong, so the noise is visible.
        let sun_dir = normalize(vec3<f32>(0.4, 0.7, 0.3));
        let sun_intensity = pow(max(0.0, dot(bounce, sun_dir)), 64.0) * 30.0;
        let sky_intensity = mix(0.3, 1.0, max(0.0, bounce.y)) * 0.6;
        let radiance = vec3<f32>(1.0, 0.95, 0.85) * sun_intensity
                     + vec3<f32>(0.5, 0.7, 1.0) * sky_intensity;
        // Cosine-weighted PDF cancels the cos in the BRDF integrand:
        // L = albedo * radiance.
        color = h.albedo * radiance;
        // Linearise depth into [0, 1] given a chosen range.
        depth = clamp(h.t / 25.0, 0.0, 1.0);
        normal = h.normal;
        albedo = h.albedo;
        roughness = h.roughness;
    } else {
        // Sky dome.
        let t = 0.5 + 0.5 * rd.y;
        color = mix(vec3<f32>(0.55, 0.7, 0.95), vec3<f32>(0.85, 0.92, 1.0), t);
        depth = 1.0;
        albedo = color;
        roughness = 1.0;
        normal = vec3<f32>(0.0, 1.0, 0.0);
    }

    textureStore(color_out, coord, vec4<f32>(color, 1.0));
    textureStore(depth_out, coord, vec4<f32>(depth, 0.0, 0.0, 0.0));
    // Write world-space normal in [0, 1] encoding (DLSS-RR is happy
    // with either normalised or re-mappable, and unorm storage forces
    // the [0, 1] range anyway).
    textureStore(normals_out, coord, vec4<f32>(normal * 0.5 + 0.5, 0.0));
    textureStore(roughness_out, coord, vec4<f32>(roughness, 0.0, 0.0, 0.0));
    textureStore(diffuse_albedo_out, coord, vec4<f32>(albedo, 1.0));
    // Specular albedo for non-metals: F0 ≈ 0.04 across RGB.
    textureStore(specular_albedo_out, coord, vec4<f32>(0.04, 0.04, 0.04, 1.0));
    // Static camera, static scene → motion vectors are zero. The
    // sub-pixel jitter is handled by DLSS-RR's `set_jitter_offsets`.
    textureStore(mv_out, coord, vec4<f32>(0.0, 0.0, 0.0, 0.0));
}
