// Camera and theme, shared by every pass.
struct Camera {
    right: vec4<f32>,
    up: vec4<f32>,
    toward: vec4<f32>,
    // xy: ground point under the screen centre.
    pivot: vec4<f32>,
    // xyz: centre of the scene, w: radius that bounds its depth.
    scene: vec4<f32>,
    // x, y: viewport in device px; z: zoom in device px per plan unit; w: thinnest stroke in px.
    view: vec4<f32>,
    // x: silhouette step in plan units; y: cosine of the crease angle; z: outline weight in plan units.
    edge: vec4<f32>,
    // x: halo falloff in plan units.
    grid: vec4<f32>,
}

struct Theme {
    palette: array<vec4<f32>, 23>,
    tints: array<vec4<f32>, 8>,
}

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<uniform> theme: Theme;

// Depth and normals from the scene pass. Read by the edge pass only.
@group(1) @binding(0) var depth_tex: texture_depth_@@MS@@2d;
@group(1) @binding(1) var normal_tex: texture_@@MS@@2d<f32>;

const SLOT_PAGE: u32 = 0u;
const SLOT_TOP: u32 = 1u;
const SLOT_SHADE0: u32 = 2u;
const SLOT_HALO: u32 = 8u;
const SLOT_GRID_MINOR: u32 = 9u;
const SLOT_GRID_MAJOR: u32 = 10u;
const SLOT_HI: u32 = 11u;
const SLOT_MID: u32 = 12u;

// Faces whose normal is steeper than this are tops.
const TOP_NZ: f32 = 0.7;
// Lines and dots sit this far toward the viewer, so they never z-fight the surface under them.
const LIFT: f32 = 0.05;
const MINOR: f32 = 8.0;
const MAJOR: f32 = 40.0;

const EDGE_DIRS: array<vec2<f32>, 8> = array<vec2<f32>, 8>(
    vec2<f32>(1.0, 0.0),
    vec2<f32>(0.70710678, 0.70710678),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(-0.70710678, 0.70710678),
    vec2<f32>(-1.0, 0.0),
    vec2<f32>(-0.70710678, -0.70710678),
    vec2<f32>(0.0, -1.0),
    vec2<f32>(0.70710678, -0.70710678),
);

struct FragOut {
    @location(0) colour: vec4<f32>,
    @location(1) normal: vec4<f32>,
}

// Clip space for a world point. Orthographic, so depth is linear along the view direction, and
// nearer points get smaller depth.
fn to_clip(p: vec3<f32>) -> vec4<f32> {
    let rel = p - vec3<f32>(camera.pivot.xy, 0.0);
    let sx = camera.view.z * dot(rel, camera.right.xyz);
    let sy = camera.view.z * dot(rel, camera.up.xyz);
    let depth = dot(p - camera.scene.xyz, camera.toward.xyz);
    return vec4<f32>(
        2.0 * sx / camera.view.x,
        2.0 * sy / camera.view.y,
        0.5 - depth / (2.0 * camera.scene.w),
        1.0,
    );
}

// The four-tone paper shade of a face, from its normal and the light that sits at the camera's
// right-hand side, so the look is the same at every azimuth.
fn paper_slot(n: vec3<f32>) -> u32 {
    if (n.z > TOP_NZ) {
        return SLOT_TOP;
    }
    let horizontal = n.xy;
    if (dot(horizontal, horizontal) < 1e-8) {
        return SLOT_TOP;
    }
    let light = -normalize(camera.right.xy);
    let lit = (dot(normalize(horizontal), light) + 1.0) * 0.5;
    return SLOT_SHADE0 + u32(clamp(floor(lit * 4.0), 0.0, 3.0));
}

// Mesh pass: one draw per kind per level, instanced.

struct MeshIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) code: u32,
    @location(3) offset: vec3<f32>,
    @location(4) tint: u32,
}

struct MeshOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) @interpolate(flat) code: u32,
    @location(2) @interpolate(flat) tint: u32,
}

@vertex
fn vs_mesh(v: MeshIn) -> MeshOut {
    var out: MeshOut;
    out.clip = to_clip(v.position + v.offset);
    out.normal = v.normal;
    out.code = v.code;
    out.tint = v.tint;
    return out;
}

@fragment
fn fs_mesh(v: MeshOut) -> FragOut {
    let n = normalize(v.normal);
    var slot = v.code - 1u;
    if (v.code == 0u) {
        slot = paper_slot(n);
    }
    let colour = theme.palette[slot] * theme.tints[v.tint];
    return FragOut(colour, vec4<f32>(n * 0.5 + 0.5, 1.0));
}

// Ground: one full-screen triangle. Each pixel's view ray meets z = 0 at a plan point, and the
// lines are analytic, so there is no geometry and no depth write.

struct FullOut {
    @builtin(position) clip: vec4<f32>,
}

fn full_clip(i: u32) -> vec4<f32> {
    let x = f32((i << 1u) & 2u) * 2.0 - 1.0;
    let y = f32(i & 2u) * 2.0 - 1.0;
    return vec4<f32>(x, y, 0.0, 1.0);
}

@vertex
fn vs_full(@builtin(vertex_index) i: u32) -> FullOut {
    return FullOut(full_clip(i));
}

// Returns (line coverage, pixels per cell) for lines every `spacing` plan units.
fn grid_cells(p: vec2<f32>, spacing: f32) -> vec2<f32> {
    let c = p / spacing;
    let d = max(fwidth(c), vec2<f32>(1e-6));
    let dist = abs(fract(c + 0.5) - 0.5) / d;
    let coverage = 1.0 - clamp(min(dist.x, dist.y), 0.0, 1.0);
    return vec2<f32>(coverage, 1.0 / max(d.x, d.y));
}

@fragment
fn fs_grid(@builtin(position) frag: vec4<f32>) -> FragOut {
    let zoom = camera.view.z;
    let sx = (frag.x - camera.view.x * 0.5) / zoom;
    let sy = (camera.view.y * 0.5 - frag.y) / zoom;
    let t = -sy * camera.up.z / camera.toward.z;
    let ground = camera.pivot.xy + sx * camera.right.xy + sy * camera.up.xy + t * camera.toward.xy;

    let minor = grid_cells(ground, MINOR);
    let major = grid_cells(ground, MAJOR);
    let fade_minor = smoothstep(2.0, 6.0, minor.y);
    let fade_major = smoothstep(1.5, 4.0, major.y);

    // Low tilt compresses the far ground into the top of the screen, so it fades out there.
    let sin_e = camera.toward.z;
    let tilt = 1.0 - smoothstep(0.25, 0.7, sin_e);
    let reach = length(camera.view.xy) * 0.5 / zoom / max(sin_e, 0.3);
    let horizon = 1.0 - tilt * smoothstep(0.25 * reach, reach, length(ground - camera.pivot.xy));

    var colour = theme.palette[SLOT_PAGE].rgb;
    colour = mix(colour, theme.palette[SLOT_GRID_MINOR].rgb, minor.x * fade_minor * horizon);
    colour = mix(colour, theme.palette[SLOT_GRID_MAJOR].rgb, major.x * fade_major * horizon);
    return FragOut(vec4<f32>(colour, 1.0), vec4<f32>(0.0));
}

// Halo: a soft ring on the ground around each footprint, drawn before the machines.

struct HaloIn {
    @location(0) position: vec3<f32>,
    @location(1) rect: vec4<f32>,
    @location(2) offset: vec3<f32>,
}

struct HaloOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) @interpolate(flat) rect: vec4<f32>,
}

@vertex
fn vs_halo(v: HaloIn) -> HaloOut {
    var out: HaloOut;
    out.clip = to_clip(v.position + v.offset);
    out.local = v.position.xy;
    out.rect = v.rect;
    return out;
}

@fragment
fn fs_halo(v: HaloOut) -> FragOut {
    let q = max(max(v.rect.xy - v.local, v.local - v.rect.zw), vec2<f32>(0.0));
    let fade = 1.0 - smoothstep(0.0, camera.grid.x, length(q));
    let halo = theme.palette[SLOT_HALO];
    return FragOut(vec4<f32>(halo.rgb, halo.a * fade * fade), vec4<f32>(0.0));
}

// Lines: each segment is six vertices. The vertex shader widens it to a constant pixel width,
// and the fragment shader anti-aliases the edge.

struct LineIn {
    @location(0) a: vec3<f32>,
    @location(1) b: vec3<f32>,
    @location(2) corner: vec2<f32>,
    @location(3) width: f32,
    @location(4) tone: u32,
    @location(5) offset: vec3<f32>,
}

struct LineOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) across: f32,
    @location(1) half_px: f32,
    @location(2) cover: f32,
    @location(3) @interpolate(flat) tone: u32,
}

@vertex
fn vs_line(v: LineIn) -> LineOut {
    let lift = camera.toward.xyz * LIFT;
    let ca = to_clip(v.a + v.offset + lift);
    let cb = to_clip(v.b + v.offset + lift);
    let view = camera.view.xy;
    let delta = (cb.xy - ca.xy) * view * 0.5;
    let len = length(delta);
    var dir = vec2<f32>(1.0, 0.0);
    if (len > 1e-6) {
        dir = delta / len;
    }
    let normal = vec2<f32>(-dir.y, dir.x);

    let width_px = v.width * camera.view.z;
    let min_px = camera.view.w;
    let stroke_px = max(width_px, min_px);
    let half_px = stroke_px * 0.5;
    let across = v.corner.y * half_px;

    let base = mix(ca, cb, v.corner.x);
    var out: LineOut;
    out.clip = vec4<f32>(base.xy + normal * across * 2.0 / view, base.z, 1.0);
    out.across = across;
    out.half_px = half_px;
    out.cover = min(width_px / min_px, 1.0);
    out.tone = v.tone;
    return out;
}

@fragment
fn fs_line(v: LineOut) -> FragOut {
    let coverage = clamp(v.half_px - abs(v.across) + 0.5, 0.0, 1.0) * v.cover;
    if (coverage <= 0.0) {
        discard;
    }
    let ink = theme.palette[v.tone];
    return FragOut(vec4<f32>(ink.rgb, coverage), vec4<f32>(0.0));
}

// Dots: a disc is a square of corners in its own plane, cut to a circle in the fragment shader.

struct DotIn {
    @location(0) centre: vec3<f32>,
    @location(1) tangent: vec3<f32>,
    @location(2) bitangent: vec3<f32>,
    @location(3) corner: vec2<f32>,
    @location(4) tone: u32,
    @location(5) offset: vec3<f32>,
}

struct DotOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) tone: u32,
}

@vertex
fn vs_dot(v: DotIn) -> DotOut {
    let p = v.centre + v.offset + camera.toward.xyz * LIFT + v.tangent * v.corner.x + v.bitangent * v.corner.y;
    var out: DotOut;
    out.clip = to_clip(p);
    out.uv = v.corner;
    out.tone = v.tone;
    return out;
}

@fragment
fn fs_dot(v: DotOut) -> FragOut {
    let r = length(v.uv);
    let aa = max(fwidth(r), 1e-4);
    let coverage = clamp((1.0 - r) / aa + 0.5, 0.0, 1.0);
    if (coverage <= 0.0) {
        discard;
    }
    let ink = theme.palette[v.tone];
    return FragOut(vec4<f32>(ink.rgb, coverage), vec4<f32>(0.0));
}

// Edge pass: a full-screen triangle that reads depth and normals around each pixel. A depth
// step is a silhouette (hi); a normal turning past the crease angle is a crease (mid). Background
// has depth 1 and no normal.

@vertex
fn vs_edge(@builtin(vertex_index) i: u32) -> FullOut {
    return FullOut(full_clip(i));
}

@fragment
fn fs_edge(@builtin(position) frag: vec4<f32>) -> @location(0) vec4<f32> {
    let dims = vec2<i32>(textureDimensions(depth_tex));
    let p = vec2<i32>(frag.xy);
    let zoom = camera.view.z;
    let min_px = camera.view.w;
    let width_px = camera.edge.z * zoom;
    let reach = max(max(width_px, min_px) * 0.5, 1.0);
    let depth_span = 2.0 * camera.scene.w;

    let c_depth = textureLoad(depth_tex, p, 0);
    let c_bg = c_depth >= 1.0;
    let c_n = textureLoad(normal_tex, p, 0).xyz * 2.0 - 1.0;

    var mark = 0u;
    for (var k = 0u; k < 8u; k = k + 1u) {
        let q = clamp(p + vec2<i32>(round(EDGE_DIRS[k] * reach)), vec2<i32>(0), dims - vec2<i32>(1));
        let s_depth = textureLoad(depth_tex, q, 0);
        let s_bg = s_depth >= 1.0;
        if (c_bg != s_bg) {
            mark = 2u;
        } else if (!c_bg) {
            // A depth step alone also fires on any slope, so it counts only where the normal turns.
            let s_n = textureLoad(normal_tex, q, 0).xyz * 2.0 - 1.0;
            if (dot(c_n, s_n) < camera.edge.y) {
                if (abs(c_depth - s_depth) * depth_span > camera.edge.x) {
                    mark = 2u;
                } else {
                    mark = max(mark, 1u);
                }
            }
        }
    }
    if (mark == 0u) {
        discard;
    }

    let cover = min(width_px / min_px, 1.0);
    if (cover <= 0.0) {
        discard;
    }
    var ink = theme.palette[SLOT_MID];
    if (mark == 2u) {
        ink = theme.palette[SLOT_HI];
    }
    return vec4<f32>(ink.rgb, ink.a * cover);
}
