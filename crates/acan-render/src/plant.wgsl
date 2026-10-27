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
    // x: depth step in plan units that makes a silhouette; y: cosine of the crease angle.
    edge: vec4<f32>,
    // x: halo falloff in plan units.
    halo: vec4<f32>,
}

struct Theme {
    palette: array<vec4<f32>, 22>,
    tints: array<vec4<f32>, 8>,
}

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<uniform> theme: Theme;

// Depth and normals from the scene pass, one value per MSAA sample. The normal's alpha is 0 on
// the ground and on surfaces lying on it, and 1 on objects. Read by the edge pass only.
@group(1) @binding(0) var depth_tex: texture_depth_@@MS@@2d;
@group(1) @binding(1) var normal_tex: texture_@@MS@@2d<f32>;

const SLOT_PAGE: u32 = 0u;
const SLOT_TOP: u32 = 1u;
const SLOT_SHADE0: u32 = 2u;
const SLOT_HALO: u32 = 8u;
const SLOT_GRID: u32 = 9u;
const SLOT_HI: u32 = 10u;
const SLOT_MID: u32 = 11u;

// Set on the code of a surface that lies on the ground. Matches `theme::FLOOR_FLAG`.
const FLOOR_FLAG: u32 = 65536u;
// MSAA samples per pixel in the scene targets: 1 or 4. The renderer fills this in.
const SAMPLES: i32 = @@SAMPLES@@;
// The grid constants of `theme.rs`, written in by the renderer.
@@GRID@@
// The ground is depth-tested this far below z = 0, so surfaces that lie on it pass the test.
const GROUND_Z: f32 = -0.05;
// Faces whose normal is steeper than this are tops.
const TOP_NZ: f32 = 0.7;
// Lines and dots sit this far toward the viewer, so they never z-fight the surface under them.
const LIFT: f32 = 0.05;
// Creases are drawn fainter than silhouettes.
const CREASE_ALPHA: f32 = 0.75;

// Clip-space depth of a world point. Nearer points get smaller depth.
fn depth_of(p: vec3<f32>) -> f32 {
    return 0.5 - dot(p - camera.scene.xyz, camera.toward.xyz) / (2.0 * camera.scene.w);
}

// Clip space for a world point. Orthographic, so depth is linear along the view direction.
fn to_clip(p: vec3<f32>) -> vec4<f32> {
    let rel = p - vec3<f32>(camera.pivot.xy, 0.0);
    let sx = camera.view.z * dot(rel, camera.right.xyz);
    let sy = camera.view.z * dot(rel, camera.up.xyz);
    return vec4<f32>(2.0 * sx / camera.view.x, 2.0 * sy / camera.view.y, depth_of(p), 1.0);
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

struct FragOut {
    @location(0) colour: vec4<f32>,
    @location(1) normal: vec4<f32>,
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
    let base = v.code & 0xffffu;
    var slot = SLOT_PAGE;
    if (base == 0u) {
        slot = paper_slot(n);
    } else {
        slot = base - 1u;
    }
    let ink = theme.palette[slot] * theme.tints[v.tint];
    var is_object = 1.0;
    if ((v.code & FLOOR_FLAG) != 0u) {
        is_object = 0.0;
    }
    return FragOut(vec4<f32>(ink.rgb, ink.a), vec4<f32>(n * 0.5 + 0.5, is_object));
}

// Ground: one full-screen triangle. Each pixel's view ray meets z = GROUND_Z at a plan point, and
// the lines are analytic, so there is no geometry. The ground writes depth and an up normal, so
// the edge pass treats it as a surface.

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

// Coverage of a 1 px anti-aliased line every `spacing` plan units, and the spacing in device px.
fn grid_line(ground: vec2<f32>, spacing: f32) -> vec2<f32> {
    let c = ground / spacing;
    let d = max(fwidth(c), vec2<f32>(1e-6));
    let dist = abs(fract(c + 0.5) - 0.5) / d;
    let coverage = 1.0 - clamp(min(dist.x, dist.y), 0.0, 1.0);
    return vec2<f32>(coverage, 1.0 / max(d.x, d.y));
}

// Ink alpha of the grid at a ground point. Each level adds its alpha while it is shown, so a line
// that two levels share is darker. The total is capped.
fn grid_alpha(ground: vec2<f32>) -> f32 {
    var sum = 0.0;
    var spacing = GRID_BASE;
    for (var level = 0; level < GRID_LEVELS; level = level + 1) {
        let line = grid_line(ground, spacing);
        let shown = smoothstep(GRID_SHOW_PX[0], GRID_SHOW_PX[1], line.y);
        sum = sum + GRID_LEVEL_ALPHA * shown * line.x;
        spacing = spacing * GRID_STEP;
    }
    return min(GRID_MAX_ALPHA, sum);
}

struct GridOut {
    @location(0) colour: vec4<f32>,
    @location(1) normal: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

@fragment
fn fs_grid(@builtin(position) frag: vec4<f32>) -> GridOut {
    let zoom = camera.view.z;
    let sx = (frag.x - camera.view.x * 0.5) / zoom;
    let sy = (camera.view.y * 0.5 - frag.y) / zoom;
    let t = -sy * camera.up.z / camera.toward.z;
    let ground = camera.pivot.xy + sx * camera.right.xy + sy * camera.up.xy + t * camera.toward.xy;

    // Fades by distance from the target in half view heights, so it looks the same at every zoom.
    let half_height = camera.view.y * 0.5 / zoom;
    let distance = length(ground - camera.pivot.xy);
    let fade = 1.0 - smoothstep(GRID_FADE_START * half_height, GRID_FADE_END * half_height, distance);
    // Thins out as the view grazes the ground.
    let grazing = smoothstep(GRID_GRAZING[0], GRID_GRAZING[1], camera.toward.z);
    let alpha = grid_alpha(ground) * fade * grazing;

    let page = theme.palette[SLOT_PAGE].rgb;
    let ink = theme.palette[SLOT_GRID].rgb;
    // Points far outside the scene's depth range clamp to its ends. No object can lie on such a
    // pixel, so the clamp never hides one.
    let depth = clamp(depth_of(vec3<f32>(ground, GROUND_Z)), 0.0, 1.0);
    return GridOut(vec4<f32>(mix(page, ink, alpha), 1.0), vec4<f32>(0.5, 0.5, 1.0, 0.0), depth);
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
    let fade = 1.0 - smoothstep(0.0, camera.halo.x, length(q));
    let halo = theme.palette[SLOT_HALO];
    return FragOut(vec4<f32>(halo.rgb, halo.a * fade * fade), vec4<f32>(0.0));
}

// Lines: each segment is six vertices. The vertex shader widens it to a constant device-pixel
// width, and the fragment shader anti-aliases the edge.

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
    let pa = v.a + v.offset + lift;
    let pb = v.b + v.offset + lift;
    let ca = to_clip(pa);
    let cb = to_clip(pb);
    let view = camera.view.xy;
    let delta = (cb.xy - ca.xy) * view * 0.5;
    let len = length(delta);
    var dir = vec2<f32>(1.0, 0.0);
    if (len > 1e-6) {
        dir = delta / len;
    }
    let normal = vec2<f32>(-dir.y, dir.x);

    let min_px = camera.view.w;
    let half_px = max(v.width, min_px) * 0.5;
    let across = v.corner.y * half_px;

    let base = mix(ca, cb, v.corner.x);
    var out: LineOut;
    out.clip = vec4<f32>(base.xy + normal * across * 2.0 / view, base.z, 1.0);
    out.across = across;
    out.half_px = half_px;
    out.cover = min(v.width / min_px, 1.0);
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

// Edge pass: a full-screen triangle that reads depth and normals at every MSAA sample. A sample
// is marked when its 1-pixel neighbour is farther away and differs in class or normal. The
// mark falls on the nearer side, so an edge is one device pixel wide. Silhouettes and contacts
// are hi; creases are mid. The coverage is the share of samples marked, which anti-aliases the
// edge at any angle.

const DIRS: array<vec2<i32>, 4> = array<vec2<i32>, 4>(
    vec2<i32>(1, 0),
    vec2<i32>(0, 1),
    vec2<i32>(-1, 0),
    vec2<i32>(0, -1),
);

struct Texel {
    depth: f32,
    normal: vec3<f32>,
    is_object: bool,
}

fn texel(p: vec2<i32>, s: i32) -> Texel {
    let n = textureLoad(normal_tex, p, s);
    return Texel(textureLoad(depth_tex, p, s), n.xyz * 2.0 - 1.0, n.w > 0.5);
}

// Returns (hi, mid) for one sample: 1 where the sample is an edge of that kind.
fn sample_marks(p: vec2<i32>, dims: vec2<i32>, s: i32) -> vec2<f32> {
    let c = texel(p, s);
    var hi = 0.0;
    var mid = 0.0;
    for (var k = 0; k < 4; k = k + 1) {
        let q = clamp(p + DIRS[k], vec2<i32>(0), dims - vec2<i32>(1));
        let o = texel(q, s);
        if (o.depth <= c.depth) {
            continue;
        }
        if (o.is_object != c.is_object) {
            hi = 1.0;
        } else if (dot(c.normal, o.normal) < camera.edge.y) {
            // Within one class, a depth step alone also fires on any slope, so it counts only
            // where the normal turns.
            if ((o.depth - c.depth) * 2.0 * camera.scene.w > camera.edge.x) {
                hi = 1.0;
            } else {
                mid = 1.0;
            }
        }
    }
    return vec2<f32>(hi, mid);
}

// True when a neighbour of sample 0 differs in class or normal, so an edge can pass this pixel.
// Most pixels lie inside one surface, and for them this one read is all the pass needs.
fn near_edge(p: vec2<i32>, dims: vec2<i32>) -> bool {
    let c = texel(p, 0);
    for (var k = 0; k < 4; k = k + 1) {
        let q = clamp(p + DIRS[k], vec2<i32>(0), dims - vec2<i32>(1));
        let o = texel(q, 0);
        if (o.is_object != c.is_object || dot(c.normal, o.normal) < camera.edge.y) {
            return true;
        }
    }
    return false;
}

@fragment
fn fs_edge(@builtin(position) frag: vec4<f32>) -> @location(0) vec4<f32> {
    let dims = vec2<i32>(textureDimensions(depth_tex));
    let p = vec2<i32>(frag.xy);
    if (!near_edge(p, dims)) {
        discard;
    }
    var hi = 0.0;
    var mid = 0.0;
    for (var s = 0; s < SAMPLES; s = s + 1) {
        let marks = sample_marks(p, dims, s);
        hi = hi + marks.x;
        mid = mid + marks.y;
    }
    hi = hi / f32(SAMPLES);
    mid = mid / f32(SAMPLES);
    if (hi <= 0.0 && mid <= 0.0) {
        discard;
    }
    if (hi > 0.0) {
        return vec4<f32>(theme.palette[SLOT_HI].rgb, hi);
    }
    return vec4<f32>(theme.palette[SLOT_MID].rgb, mid * CREASE_ALPHA);
}
