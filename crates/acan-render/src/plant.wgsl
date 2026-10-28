// Camera and theme, shared by every pass.
struct Camera {
    right: vec4<f32>,
    up: vec4<f32>,
    toward: vec4<f32>,
    // xyz: the eye. For the orthographic view it is the equivalent perspective eye, used for fog.
    // w: 1 when orthographic.
    eye: vec4<f32>,
    // xy: the ground pivot. z: eye distance, the length from the eye to the pivot.
    pivot: vec4<f32>,
    // x, y: viewport in device px. z: zoom in device px per plan unit at the pivot. w: thinnest
    // stroke in device px.
    view: vec4<f32>,
    // x: near plane. y: far plane (orthographic only). z: focal length in device px (perspective).
    depth: vec4<f32>,
    // x, y: fog starts and ends, in eye distances. z, w: the angle fade, as the absolute sine of a
    // ray's elevation, from and to.
    fog: vec4<f32>,
}

struct Theme {
    palette: array<vec4<f32>, @@PALETTE_LEN@@>,
    tints: array<vec4<f32>, 8>,
}

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<uniform> theme: Theme;

// Depth and object IDs from the scene pass, one value per MSAA sample. The ID's alpha holds the
// fog of the surface. Read by the edge pass only.
@group(1) @binding(0) var depth_tex: texture_depth_@@MS@@2d;
@group(1) @binding(1) var id_tex: texture_@@MS@@2d<f32>;

// MSAA samples per pixel in the scene targets: 1 or 4. The renderer fills this in.
const SAMPLES: i32 = @@SAMPLES@@;
// Palette slots and tuning constants from `theme.rs`, written in by the renderer.
@@THEME@@
// The ground is depth-tested this far below z = 0, so surfaces that lie on it pass the test.
const GROUND_Z: f32 = -0.05;
// Faces whose normal is steeper than this are tops.
const TOP_NZ: f32 = 0.7;
// Lines and dots sit this far toward the viewer, so they never z-fight the surface under them.
const LIFT: f32 = 0.05;

// Depth is reverse: 1 at the near plane and smaller farther away, cleared to 0. Perspective puts
// the far plane at infinity, so the depth is near / distance.
fn is_ortho() -> bool {
    return camera.eye.w > 0.5;
}

// Distance in front of the eye, along the view direction.
fn depth_of(p: vec3<f32>) -> f32 {
    return -dot(p - camera.eye.xyz, camera.toward.xyz);
}

fn ndc_depth(p: vec3<f32>) -> f32 {
    let d = depth_of(p);
    if (is_ortho()) {
        return 1.0 - (d - camera.depth.x) / (camera.depth.y - camera.depth.x);
    }
    return camera.depth.x / d;
}

// Clip space for a world point. Perspective divides by the depth, so the screen position is the
// focal length times the sideways offset over the depth.
fn to_clip(p: vec3<f32>) -> vec4<f32> {
    if (is_ortho()) {
        let rel = p - vec3<f32>(camera.pivot.xy, 0.0);
        let x = 2.0 * camera.view.z * dot(rel, camera.right.xyz) / camera.view.x;
        let y = 2.0 * camera.view.z * dot(rel, camera.up.xyz) / camera.view.y;
        return vec4<f32>(x, y, ndc_depth(p), 1.0);
    }
    let rel = p - camera.eye.xyz;
    let x = 2.0 * camera.depth.z * dot(rel, camera.right.xyz) / camera.view.x;
    let y = 2.0 * camera.depth.z * dot(rel, camera.up.xyz) / camera.view.y;
    return vec4<f32>(x, y, camera.depth.x, depth_of(p));
}

// Fog from the eye: nothing fades nearer than the fog start, and the far end is page white.
// Distances are in eye distances from the pivot, so the look holds at every zoom.
fn fog_at(p: vec3<f32>) -> f32 {
    let d = camera.pivot.z;
    return smoothstep(camera.fog.x * d, camera.fog.y * d, distance(p, camera.eye.xyz));
}

// The paper grey of a face, from its normal. The light is fixed to the camera's left, so a face
// keeps its tone at every azimuth: tops are nearly white, sides run from the lit tone to the dark
// one as they turn away from the light, and slopes blend toward the top.
fn paper_grey(n: vec3<f32>) -> f32 {
    let horizontal = n.xy;
    var side = PAPER_LIT;
    if (dot(horizontal, horizontal) > 1e-8) {
        let light = -normalize(camera.right.xy);
        let away = 0.5 - 0.5 * dot(normalize(horizontal), light);
        side = mix(PAPER_LIT, PAPER_DARK, away);
    }
    let up = smoothstep(0.3, TOP_NZ + 0.25, abs(n.z));
    return 1.0 - mix(side, PAPER_TOP, up);
}

// An object ID in three 8-bit channels. The alpha carries the fog for the outline pass.
fn id_colour(id: u32, fog: f32) -> vec4<f32> {
    let bytes = vec3<f32>(f32(id & 255u), f32((id >> 8u) & 255u), f32((id >> 16u) & 255u));
    return vec4<f32>(bytes / 255.0, fog);
}

struct FragOut {
    @location(0) colour: vec4<f32>,
    @location(1) id: vec4<f32>,
}

// Mesh pass: one draw per kind per level, instanced.

struct MeshIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) code: u32,
    @location(3) offset: vec3<f32>,
    @location(4) tint: u32,
    @location(5) id: u32,
}

struct MeshOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) @interpolate(flat) code: u32,
    @location(2) @interpolate(flat) tint: u32,
    @location(3) world: vec3<f32>,
    @location(4) @interpolate(flat) id: u32,
}

@vertex
fn vs_mesh(v: MeshIn) -> MeshOut {
    var out: MeshOut;
    let p = v.position + v.offset;
    out.clip = to_clip(p);
    out.normal = v.normal;
    out.code = v.code;
    out.tint = v.tint;
    out.world = p;
    out.id = v.id;
    return out;
}

@fragment
fn fs_mesh(v: MeshOut) -> FragOut {
    let n = normalize(v.normal);
    var ink = vec4<f32>(vec3<f32>(paper_grey(n)), 1.0);
    if (v.code != 0u) {
        ink = theme.palette[v.code - 1u];
    }
    ink = ink * theme.tints[v.tint];
    let fog = fog_at(v.world);
    let page = theme.palette[SLOT_PAGE].rgb;
    return FragOut(vec4<f32>(mix(ink.rgb, page, fog), ink.a), id_colour(v.id, fog));
}

// Ground: one full-screen triangle. Each pixel's view ray meets the ground at a plan point, and
// the grid lines are analytic, so there is no geometry. A ray that rises above the horizon sees
// no ground: it is page white and writes no depth.

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

struct Ground {
    point: vec2<f32>,
    // The unit direction of the view ray through the pixel.
    dir: vec3<f32>,
    hit: bool,
}

// The ground point under a device pixel, and whether the pixel sees the ground at all.
fn ground_ray(frag: vec2<f32>) -> Ground {
    let sx = frag.x - camera.view.x * 0.5;
    let sy = camera.view.y * 0.5 - frag.y;
    if (is_ortho()) {
        let ox = sx / camera.view.z;
        let oy = sy / camera.view.z;
        let t = -oy * camera.up.z / camera.toward.z;
        let g = camera.pivot.xy + ox * camera.right.xy + oy * camera.up.xy + t * camera.toward.xy;
        return Ground(g, -camera.toward.xyz, true);
    }
    let dir = -camera.depth.z * camera.toward.xyz + sx * camera.right.xyz + sy * camera.up.xyz;
    let t = -camera.eye.z / min(dir.z, -1e-6);
    let g = camera.eye.xy + t * dir.xy;
    return Ground(g, normalize(dir), dir.z < 0.0);
}

// Coverage of a 1 px anti-aliased line every `spacing` plan units, and the spacing in device px.
fn grid_line(ground: vec2<f32>, spacing: f32) -> vec2<f32> {
    let c = ground / spacing;
    let d = max(fwidth(c), vec2<f32>(1e-6));
    let dist = abs(fract(c + 0.5) - 0.5) / d;
    let coverage = 1.0 - clamp(min(dist.x, dist.y), 0.0, 1.0);
    return vec2<f32>(coverage, 1.0 / max(d.x, d.y));
}

// Ink alpha of the grid at a ground point. A level fades in as its on-screen spacing grows,
// starting minor, and turns major as the next finer level fades in under it. The strongest
// level wins, so a line that two levels share is no darker than a major line.
fn grid_alpha(ground: vec2<f32>) -> f32 {
    var best = 0.0;
    var spacing = GRID_BASE;
    for (var level = 0; level < GRID_LEVELS; level = level + 1) {
        let line = grid_line(ground, spacing);
        let shown = smoothstep(GRID_SHOW_PX[0], GRID_SHOW_PX[1], line.y);
        let finer = smoothstep(GRID_SHOW_PX[0], GRID_SHOW_PX[1], line.y / GRID_STEP);
        let ink = mix(GRID_MINOR_ALPHA, GRID_MAJOR_ALPHA, finer);
        best = max(best, ink * shown * line.x);
        spacing = spacing * GRID_STEP;
    }
    return best;
}

struct GridOut {
    @location(0) colour: vec4<f32>,
    @location(1) id: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

@fragment
fn fs_grid(@builtin(position) frag: vec4<f32>) -> GridOut {
    let ray = ground_ray(frag.xy);
    let g = ray.point;
    // Fog and the angle fade are computed for every pixel, then the sky is selected away, so the
    // derivatives in the grid lines stay in uniform control flow.
    let fog = fog_at(vec3<f32>(g, 0.0));
    let graze = smoothstep(camera.fog.z, camera.fog.w, abs(ray.dir.z));
    let line_alpha = grid_alpha(g) * graze * (1.0 - fog);
    let alpha = select(0.0, line_alpha, ray.hit);

    let page = theme.palette[SLOT_PAGE].rgb;
    let ink = theme.palette[SLOT_GRID].rgb;
    let depth = select(0.0, clamp(ndc_depth(vec3<f32>(g, GROUND_Z)), 0.0, 1.0), ray.hit);
    return GridOut(vec4<f32>(mix(page, ink, alpha), 1.0), vec4<f32>(0.0), depth);
}

// Belts: flat opaque ribbons on the ground. The side and end lines are drawn in the fill, from the
// distance to the ribbon's border in device pixels, so they never break up at a distance. They
// fade as the ribbon narrows on screen.

struct RibbonIn {
    @location(0) position: vec2<f32>,
    @location(1) side: f32,
    @location(2) along: f32,
    @location(3) length: f32,
    @location(4) half: f32,
}

struct RibbonOut {
    @builtin(position) clip: vec4<f32>,
    // Plan units from the centre line, signed.
    @location(0) across: f32,
    @location(1) along: f32,
    @location(2) @interpolate(flat) size: vec2<f32>,
    @location(3) world: vec3<f32>,
}

@vertex
fn vs_ribbon(v: RibbonIn) -> RibbonOut {
    let p = vec3<f32>(v.position, 0.0);
    var out: RibbonOut;
    out.clip = to_clip(p);
    out.across = v.side * v.half;
    out.along = v.along;
    out.size = vec2<f32>(v.length, v.half);
    out.world = p;
    return out;
}

@fragment
fn fs_ribbon(v: RibbonOut) -> FragOut {
    let per_px_across = max(fwidth(v.across), 1e-6);
    let per_px_along = max(fwidth(v.along), 1e-6);
    let side_px = (v.size.y - abs(v.across)) / per_px_across;
    let end_px = min(v.along, v.size.x - v.along) / per_px_along;
    let line = clamp(BELT_EDGE_PX + 0.5 - min(side_px, end_px), 0.0, 1.0);
    let width_px = 2.0 * v.size.y / per_px_across;
    let shown = smoothstep(BELT_EDGE_FADE_PX[0], BELT_EDGE_FADE_PX[1], width_px);
    let edge = theme.palette[SLOT_BELT_EDGE];
    let fill = theme.palette[SLOT_BELT].rgb;
    let colour = mix(fill, edge.rgb, edge.a * line * shown);
    let fog = fog_at(v.world);
    let page = theme.palette[SLOT_PAGE].rgb;
    return FragOut(vec4<f32>(mix(colour, page, fog), 1.0), vec4<f32>(0.0));
}

// Contact shadow: a short soft falloff on the ground around each footprint, pushed a little
// away from the light, drawn before the machines.

struct HaloIn {
    @location(0) position: vec3<f32>,
    @location(1) rect: vec4<f32>,
    @location(2) offset: vec3<f32>,
}

struct HaloOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) @interpolate(flat) rect: vec4<f32>,
    @location(2) world: vec3<f32>,
}

@vertex
fn vs_halo(v: HaloIn) -> HaloOut {
    var out: HaloOut;
    let p = v.position + v.offset;
    out.clip = to_clip(p);
    out.local = v.position.xy;
    out.rect = v.rect;
    out.world = p;
    return out;
}

@fragment
fn fs_halo(v: HaloOut) -> FragOut {
    let shifted = v.local - SHADOW_SHIFT * normalize(camera.right.xy);
    let q = max(max(v.rect.xy - shifted, shifted - v.rect.zw), vec2<f32>(0.0));
    let fade = 1.0 - smoothstep(0.0, SHADOW_REACH, length(q));
    let halo = theme.palette[SLOT_SHADOW];
    let page = theme.palette[SLOT_PAGE].rgb;
    let fog = fog_at(v.world);
    let alpha = halo.a * fade * fade * (1.0 - fog);
    return FragOut(vec4<f32>(mix(halo.rgb, page, fog), alpha), vec4<f32>(0.0));
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
    @location(4) world: vec3<f32>,
}

// How much of a machine's inner line art shows, from its size on screen at its origin.
fn detail_shown(origin: vec3<f32>) -> f32 {
    var px_per_unit = camera.view.z;
    if (!is_ortho()) {
        px_per_unit = camera.depth.z / max(depth_of(origin), camera.depth.x);
    }
    return smoothstep(LOD_FADE_PX[0], LOD_FADE_PX[1], px_per_unit * LOD_REFERENCE);
}

@vertex
fn vs_line(v: LineIn) -> LineOut {
    let shown = detail_shown(v.offset);
    if (shown <= 0.0) {
        // Every corner of the segment lands on one point, so it rasterizes nothing.
        var gone: LineOut;
        gone.clip = vec4<f32>(0.0, 0.0, 0.0, 1.0);
        return gone;
    }
    let lift = camera.toward.xyz * LIFT;
    let pa = v.a + v.offset + lift;
    let pb = v.b + v.offset + lift;
    let ca = to_clip(pa);
    let cb = to_clip(pb);
    let view = camera.view.xy;
    // The stroke's direction on screen, in device px, from the ends after the perspective divide.
    let delta = (cb.xy / cb.w - ca.xy / ca.w) * view * 0.5;
    let len = length(delta);
    var dir = vec2<f32>(1.0, 0.0);
    if (len > 1e-6) {
        dir = delta / len;
    }
    let normal = vec2<f32>(-dir.y, dir.x);

    let min_px = camera.view.w;
    let half_px = max(v.width, min_px) * 0.5;
    let across = v.corner.y * half_px;

    // Clip space is linear along a segment, so the interpolated point is exact. An offset of
    // `across` px is 2 / view times that in NDC, and NDC is clip over w.
    let base = mix(ca, cb, v.corner.x);
    let shift = normal * across * 2.0 / view * base.w;
    var out: LineOut;
    out.clip = vec4<f32>(base.xy + shift, base.z, base.w);
    out.across = across;
    out.half_px = half_px;
    out.cover = min(v.width / min_px, 1.0) * shown;
    out.tone = v.tone;
    out.world = mix(pa, pb, v.corner.x);
    return out;
}

@fragment
fn fs_line(v: LineOut) -> FragOut {
    let coverage = clamp(v.half_px - abs(v.across) + 0.5, 0.0, 1.0) * v.cover;
    if (coverage <= 0.0) {
        discard;
    }
    let ink = theme.palette[v.tone];
    let page = theme.palette[SLOT_PAGE].rgb;
    let fog = fog_at(v.world);
    return FragOut(vec4<f32>(mix(ink.rgb, page, fog), ink.a * coverage), vec4<f32>(0.0));
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
    @location(2) world: vec3<f32>,
}

@vertex
fn vs_dot(v: DotIn) -> DotOut {
    let p = v.centre + v.offset + camera.toward.xyz * LIFT + v.tangent * v.corner.x + v.bitangent * v.corner.y;
    var out: DotOut;
    out.clip = to_clip(p);
    out.uv = v.corner;
    out.tone = v.tone;
    out.world = p;
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
    let page = theme.palette[SLOT_PAGE].rgb;
    let fog = fog_at(v.world);
    return FragOut(vec4<f32>(mix(ink.rgb, page, fog), ink.a * coverage), vec4<f32>(0.0));
}

// Outline pass: a full-screen triangle that reads the object ID and depth at every MSAA sample.
// A sample is on the outline when a 1-pixel neighbour is another object that lies farther away,
// or is the ground. The mark falls on the object's own side of the edge, so the line is one
// device pixel wide. Its coverage is the share of samples marked, which anti-aliases it.

const DIRS: array<vec2<i32>, 4> = array<vec2<i32>, 4>(
    vec2<i32>(1, 0),
    vec2<i32>(0, 1),
    vec2<i32>(-1, 0),
    vec2<i32>(0, -1),
);

struct Texel {
    id: u32,
    depth: f32,
    fog: f32,
}

fn texel(p: vec2<i32>, s: i32) -> Texel {
    let c = textureLoad(id_tex, p, s);
    let bytes = vec3<u32>(round(c.rgb * 255.0));
    let id = bytes.r | (bytes.g << 8u) | (bytes.b << 16u);
    return Texel(id, textureLoad(depth_tex, p, s), c.a);
}

// 1 where sample `s` of pixel `p` is on the outline, else 0. The ground (ID 0) is never outlined.
fn sample_mark(p: vec2<i32>, dims: vec2<i32>, s: i32) -> f32 {
    let c = texel(p, s);
    if (c.id == 0u) {
        return 0.0;
    }
    for (var k = 0; k < 4; k = k + 1) {
        let q = clamp(p + DIRS[k], vec2<i32>(0), dims - vec2<i32>(1));
        let o = texel(q, s);
        if (o.id != c.id && (o.id == 0u || o.depth < c.depth)) {
            return 1.0;
        }
    }
    return 0.0;
}

@fragment
fn fs_edge(@builtin(position) frag: vec4<f32>) -> @location(0) vec4<f32> {
    let dims = vec2<i32>(textureDimensions(depth_tex));
    let p = vec2<i32>(frag.xy);
    // Most pixels lie inside one object, and their sample-0 neighbours share its ID. Those skip
    // the per-sample work.
    let centre = texel(p, 0);
    var interior = true;
    for (var k = 0; k < 4; k = k + 1) {
        let q = clamp(p + DIRS[k], vec2<i32>(0), dims - vec2<i32>(1));
        if (texel(q, 0).id != centre.id) {
            interior = false;
        }
    }
    if (interior) {
        discard;
    }
    var cover = 0.0;
    for (var s = 0; s < SAMPLES; s = s + 1) {
        cover = cover + sample_mark(p, dims, s);
    }
    cover = cover / f32(SAMPLES);
    if (cover <= 0.0) {
        discard;
    }
    let outline = theme.palette[SLOT_OUTLINE];
    let page = theme.palette[SLOT_PAGE].rgb;
    return vec4<f32>(mix(outline.rgb, page, centre.fog), outline.a * cover);
}
