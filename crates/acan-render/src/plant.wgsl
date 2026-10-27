// Fills, strokes and dots of the plant. Every item has its own depth order (`order`
// inside its machine, `rank` for the machine), so the depth buffer resolves painter
// order and whole model kinds draw in one call. Vertices are device pixels after
// `to_px`. Strokes and dots widen to at least `min_px` and fade below it.

struct Camera {
    center: vec2<f32>,
    viewport: vec2<f32>,
    zoom: f32,
    min_px: f32,
    depth_step: f32,
    pad: f32,
};

struct Theme {
    palette: array<vec4<f32>, 22>,
    tints: array<vec4<f32>, 8>,
};

@group(0) @binding(0) var<uniform> cam: Camera;
@group(0) @binding(1) var<uniform> theme: Theme;

fn to_px(p: vec2<f32>) -> vec2<f32> {
    return (p - cam.center) * cam.zoom + cam.viewport * 0.5;
}

fn to_clip(px: vec2<f32>, z: f32) -> vec4<f32> {
    return vec4<f32>(px.x / cam.viewport.x * 2.0 - 1.0, 1.0 - px.y / cam.viewport.y * 2.0, z, 1.0);
}

fn depth_of(rank: f32, order: f32) -> f32 {
    return 1.0 - (rank + order) * cam.depth_step;
}

fn paint(tone: u32, tint: u32, alpha: f32) -> vec4<f32> {
    let c = theme.palette[tone] * theme.tints[tint];
    return vec4<f32>(c.rgb, c.a * alpha);
}

struct FillIn {
    @location(0) pos: vec2<f32>,
    @location(1) alpha: f32,
    @location(2) tone: u32,
    @location(3) order: f32,
    @location(4) offset: vec2<f32>,
    @location(5) rank: f32,
    @location(6) tint: u32,
};

struct Paint {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_fill(v: FillIn) -> Paint {
    var o: Paint;
    o.clip = to_clip(to_px(v.offset + v.pos), depth_of(v.rank, v.order));
    o.color = paint(v.tone, v.tint, v.alpha);
    return o;
}

@fragment
fn fs_fill(i: Paint) -> @location(0) vec4<f32> {
    return i.color;
}

struct LineIn {
    @location(0) a: vec2<f32>,
    @location(1) b: vec2<f32>,
    @location(2) corner: vec2<f32>,
    @location(3) width: f32,
    @location(4) tone: u32,
    @location(5) order: f32,
    @location(6) offset: vec2<f32>,
    @location(7) rank: f32,
    @location(8) tint: u32,
};

struct LineOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) side: f32,
    @location(2) half_px: f32,
};

@vertex
fn vs_line(v: LineIn) -> LineOut {
    let pa = to_px(v.offset + v.a);
    let pb = to_px(v.offset + v.b);
    let span = pb - pa;
    let len = length(span);
    let dir = select(vec2<f32>(1.0, 0.0), span / max(len, 1e-6), len > 1e-6);
    let natural = v.width * cam.zoom;
    let half_px = max(natural, cam.min_px) * 0.5;
    let along = mix(pa, pb, v.corner.x) + dir * (2.0 * v.corner.x - 1.0) * half_px;
    let px = along + vec2<f32>(-dir.y, dir.x) * v.corner.y * half_px;
    var o: LineOut;
    o.clip = to_clip(px, depth_of(v.rank, v.order));
    o.color = paint(v.tone, v.tint, clamp(natural / cam.min_px, 0.0, 1.0));
    o.side = v.corner.y * half_px;
    o.half_px = half_px;
    return o;
}

@fragment
fn fs_line(i: LineOut) -> @location(0) vec4<f32> {
    let cov = clamp(i.half_px - abs(i.side) + 0.5, 0.0, 1.0);
    return vec4<f32>(i.color.rgb, i.color.a * cov);
}

struct DotIn {
    @location(0) center: vec2<f32>,
    @location(1) corner: vec2<f32>,
    @location(2) radius: f32,
    @location(3) tone: u32,
    @location(4) order: f32,
    @location(5) offset: vec2<f32>,
    @location(6) rank: f32,
    @location(7) tint: u32,
};

struct DotOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local: vec2<f32>,
    @location(2) radius_px: f32,
};

@vertex
fn vs_dot(v: DotIn) -> DotOut {
    let natural = v.radius * cam.zoom;
    let r = max(natural, cam.min_px * 0.5);
    let local = v.corner * (r + 1.0);
    var o: DotOut;
    o.clip = to_clip(to_px(v.offset + v.center) + local, depth_of(v.rank, v.order));
    o.color = paint(v.tone, v.tint, clamp(natural / (cam.min_px * 0.5), 0.0, 1.0));
    o.local = local;
    o.radius_px = r;
    return o;
}

@fragment
fn fs_dot(i: DotOut) -> @location(0) vec4<f32> {
    let cov = clamp(i.radius_px - length(i.local) + 0.5, 0.0, 1.0);
    return vec4<f32>(i.color.rgb, i.color.a * cov);
}
