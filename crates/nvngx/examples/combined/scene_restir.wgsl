// Render-resolution scene shader for the combined DLSS pipeline demo.
//
// Same analytic scene as the standalone DLSS-RR example (checker
// plane + three colored spheres + sky) but additionally:
//   - emits per-pixel diffuse and specular hit-distance buffers for
//     the ReSTIR-style DLSS-RR variant;
//   - computes proper geometric motion vectors based on the camera
//     orbiting between frames (`current_angle` vs `prev_angle`).
//
// MVs are jitter-free (DLSS-RR's `set_jitter_offsets` handles that
// separately) but pixel-space.

@group(0) @binding(0) var color_out: texture_storage_2d<rgba16float, write>;
@group(0) @binding(1) var depth_out: texture_storage_2d<r32float, write>;
@group(0) @binding(2) var normals_out: texture_storage_2d<rgba16float, write>;
@group(0) @binding(3) var roughness_out: texture_storage_2d<r8unorm, write>;
@group(0) @binding(4) var diffuse_albedo_out: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(5) var specular_albedo_out: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(6) var mv_out: texture_storage_2d<rg16float, write>;
@group(0) @binding(7) var diffuse_hit_dist_out: texture_storage_2d<r32float, write>;
@group(0) @binding(8) var specular_hit_dist_out: texture_storage_2d<r32float, write>;

struct Params {
    canvas_size: vec2<u32>,
    jitter: vec2<f32>,
    frame_index: u32,
    current_angle: f32,
    prev_angle: f32,
    _pad: u32,
}

@group(0) @binding(9) var<uniform> pc: Params;

const PI: f32 = 3.14159265358979;
const CAMERA_DIST: f32 = 5.0;
const CAMERA_HEIGHT: f32 = 1.5;
const CAMERA_LOOK_Y: f32 = 1.0;
const FOV_HALF: f32 = 0.45;

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

fn cosine_hemisphere(n: vec3<f32>, rng: vec2<f32>) -> vec3<f32> {
    let r = sqrt(rng.x);
    let phi = 2.0 * PI * rng.y;
    let local = vec3<f32>(r * cos(phi), r * sin(phi), sqrt(max(0.0, 1.0 - rng.x)));
    let up = select(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), abs(n.y) < 0.999);
    let t = normalize(cross(up, n));
    let b = cross(n, t);
    return normalize(local.x * t + local.y * b + local.z * n);
}

struct Camera {
    pos: vec3<f32>,
    right: vec3<f32>,
    up: vec3<f32>,
    fwd: vec3<f32>,
}

fn make_camera(angle: f32) -> Camera {
    let pos = vec3<f32>(sin(angle) * CAMERA_DIST, CAMERA_HEIGHT, cos(angle) * CAMERA_DIST);
    let look = vec3<f32>(0.0, CAMERA_LOOK_Y, 0.0);
    let world_up = vec3<f32>(0.0, 1.0, 0.0);
    let fwd = normalize(look - pos);
    let right = normalize(cross(fwd, world_up));
    let up = cross(right, fwd);
    return Camera(pos, right, up, fwd);
}

// Project a world point through the camera, returning pixel-space
// screen coordinates ((-1, -1) if behind camera).
fn project(world: vec3<f32>, c: Camera, size: vec2<f32>) -> vec2<f32> {
    let rel = world - c.pos;
    let x = dot(c.right, rel);
    let y = dot(c.up, rel);
    let z = dot(c.fwd, rel);
    if (z <= 0.0001) { return vec2<f32>(-1.0, -1.0); }
    let aspect = size.x / size.y;
    let fov_tan = tan(FOV_HALF);
    let ndc_x = (x / z) / (aspect * fov_tan);
    let ndc_y = (y / z) / fov_tan;
    return vec2<f32>(
        (ndc_x * 0.5 + 0.5) * size.x,
        (1.0 - (ndc_y * 0.5 + 0.5)) * size.y,
    );
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

fn intersect_plane(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    if (abs(rd.y) < 1e-4) { return -1.0; }
    let t = -ro.y / rd.y;
    return select(-1.0, t, t > 0.001);
}

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

    let tp = intersect_plane(ro, rd);
    if (tp > 0.0 && tp < best_t) {
        best_t = tp;
        let p = ro + rd * tp;
        let cx = i32(floor(p.x * 0.5));
        let cz = i32(floor(p.z * 0.5));
        let checker = ((cx + cz) & 1) == 0;
        let alb = select(vec3<f32>(0.18), vec3<f32>(0.55), checker);
        h = Hit(tp, p, vec3<f32>(0.0, 1.0, 0.0), alb, 1.0);
    }

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
    let size_i = vec2<i32>(i32(pc.canvas_size.x), i32(pc.canvas_size.y));
    let coord = vec2<i32>(i32(gid.x), i32(gid.y));
    if (coord.x >= size_i.x || coord.y >= size_i.y) { return; }

    let cam = make_camera(pc.current_angle);
    let cam_prev = make_camera(pc.prev_angle);

    let size_f = vec2<f32>(size_i);
    let aspect = size_f.x / size_f.y;
    let fov_tan = tan(FOV_HALF);
    let uv = (vec2<f32>(coord) + vec2<f32>(0.5) + pc.jitter) / size_f;
    let ndc = vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
    let rd = normalize(
        cam.fwd
        + cam.right * (ndc.x * aspect * fov_tan)
        + cam.up * (ndc.y * fov_tan)
    );
    let ro = cam.pos;

    let h = trace_scene(ro, rd);

    var color: vec3<f32>;
    var depth: f32 = 1.0;
    var normal: vec3<f32> = vec3<f32>(0.0, 1.0, 0.0);
    var albedo: vec3<f32> = vec3<f32>(0.05);
    var roughness: f32 = 1.0;
    var mv: vec2<f32> = vec2<f32>(0.0, 0.0);
    var hit_dist: f32 = 0.0;

    if (h.t > 0.0) {
        // 1-spp diffuse Monte Carlo against a sky-dome + sun.
        let rng = rand2(vec2<u32>(u32(coord.x), u32(coord.y)), pc.frame_index);
        let bounce = cosine_hemisphere(h.normal, rng);
        let sun_dir = normalize(vec3<f32>(0.4, 0.7, 0.3));
        let sun_intensity = pow(max(0.0, dot(bounce, sun_dir)), 64.0) * 30.0;
        let sky_intensity = mix(0.3, 1.0, max(0.0, bounce.y)) * 0.6;
        let radiance = vec3<f32>(1.0, 0.95, 0.85) * sun_intensity
                     + vec3<f32>(0.5, 0.7, 1.0) * sky_intensity;
        color = h.albedo * radiance;
        depth = clamp(h.t / 25.0, 0.0, 1.0);
        normal = h.normal;
        albedo = h.albedo;
        roughness = h.roughness;
        hit_dist = h.t;

        // Geometric MV from camera motion alone (no jitter).
        let cam_unjitter_uv = (vec2<f32>(coord) + vec2<f32>(0.5)) / size_f;
        let cur_screen = vec2<f32>(
            cam_unjitter_uv.x * size_f.x,
            cam_unjitter_uv.y * size_f.y,
        );
        let prev_screen = project(h.pos, cam_prev, size_f);
        // If prev_screen is invalid (behind camera), zero the MV.
        if (prev_screen.x >= 0.0) {
            mv = cur_screen - prev_screen;
        }
    } else {
        let t = 0.5 + 0.5 * rd.y;
        color = mix(vec3<f32>(0.55, 0.7, 0.95), vec3<f32>(0.85, 0.92, 1.0), t);
        depth = 1.0;
        albedo = color;
        roughness = 1.0;
        normal = vec3<f32>(0.0, 1.0, 0.0);
    }

    textureStore(color_out, coord, vec4<f32>(color, 1.0));
    textureStore(depth_out, coord, vec4<f32>(depth, 0.0, 0.0, 0.0));
    textureStore(normals_out, coord, vec4<f32>(normal * 0.5 + 0.5, 0.0));
    textureStore(roughness_out, coord, vec4<f32>(roughness, 0.0, 0.0, 0.0));
    textureStore(diffuse_albedo_out, coord, vec4<f32>(albedo, 1.0));
    textureStore(specular_albedo_out, coord, vec4<f32>(0.04, 0.04, 0.04, 1.0));
    textureStore(mv_out, coord, vec4<f32>(mv, 0.0, 0.0));
    textureStore(diffuse_hit_dist_out, coord, vec4<f32>(hit_dist, 0.0, 0.0, 0.0));
    // Specular hit distance: mirror the diffuse one. In a real ReSTIR
    // pipeline this would be a separate specular bounce; the demo
    // doesn't trace one, so we reuse the primary hit distance as a
    // stand-in.
    textureStore(specular_hit_dist_out, coord, vec4<f32>(hit_dist, 0.0, 0.0, 0.0));
}
