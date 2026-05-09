// Target-resolution depth + motion-vector pass for DLSS-G.
//
// DLSS-G expects depth and MVs at the same resolution as the
// backbuffer (= upscaled DLSS-RR output). The render-resolution
// outputs `scene_restir.wgsl` produces are too small. Re-trace the
// scene at target resolution but write only depth and MVs.

@group(0) @binding(0) var depth_out: texture_storage_2d<r32float, write>;
@group(0) @binding(1) var mv_out: texture_storage_2d<rg16float, write>;

struct Params {
    canvas_size: vec2<u32>,
    _pad0: vec2<f32>,
    _frame_index: u32,
    current_angle: f32,
    prev_angle: f32,
    _pad1: u32,
}

@group(0) @binding(2) var<uniform> pc: Params;

const CAMERA_DIST: f32 = 5.0;
const CAMERA_HEIGHT: f32 = 1.5;
const CAMERA_LOOK_Y: f32 = 1.0;
const FOV_HALF: f32 = 0.45;

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

struct TraceResult {
    hit: bool,
    t: f32,
    pos: vec3<f32>,
}

fn trace_scene(ro: vec3<f32>, rd: vec3<f32>) -> TraceResult {
    var best_t: f32 = 1e30;
    var best_pos = vec3<f32>(0.0);
    var any_hit = false;

    let tp = intersect_plane(ro, rd);
    if (tp > 0.0 && tp < best_t) {
        best_t = tp;
        best_pos = ro + rd * tp;
        any_hit = true;
    }

    let spheres = array<vec4<f32>, 3>(
        vec4<f32>(-1.5, 1.0, 0.0, 1.0),
        vec4<f32>(0.0,  1.0, 0.0, 1.0),
        vec4<f32>(1.5,  1.0, 0.0, 1.0),
    );
    for (var i = 0u; i < 3u; i++) {
        let s = spheres[i];
        let ts = intersect_sphere(ro, rd, s.xyz, s.w);
        if (ts > 0.0 && ts < best_t) {
            best_t = ts;
            best_pos = ro + rd * ts;
            any_hit = true;
        }
    }

    return TraceResult(any_hit, best_t, best_pos);
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
    let uv = (vec2<f32>(coord) + vec2<f32>(0.5)) / size_f;
    let ndc = vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
    let rd = normalize(
        cam.fwd
        + cam.right * (ndc.x * aspect * fov_tan)
        + cam.up * (ndc.y * fov_tan)
    );

    let h = trace_scene(cam.pos, rd);

    var depth: f32 = 1.0;
    var mv: vec2<f32> = vec2<f32>(0.0, 0.0);

    if (h.hit) {
        depth = clamp(h.t / 25.0, 0.0, 1.0);
        let cur_screen = vec2<f32>(uv.x * size_f.x, uv.y * size_f.y);
        let prev_screen = project(h.pos, cam_prev, size_f);
        if (prev_screen.x >= 0.0) {
            mv = cur_screen - prev_screen;
        }
    }

    textureStore(depth_out, coord, vec4<f32>(depth, 0.0, 0.0, 0.0));
    textureStore(mv_out, coord, vec4<f32>(mv, 0.0, 0.0));
}
