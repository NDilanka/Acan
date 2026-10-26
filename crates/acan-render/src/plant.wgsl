struct Camera {
    center: vec2<f32>,
    scale: vec2<f32>,
};

@group(0) @binding(0) var<uniform> camera: Camera;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec4<f32>,
};

fn to_clip(world: vec2<f32>) -> vec4<f32> {
    return vec4<f32>((world - camera.center) * camera.scale, 0.0, 1.0);
}

@vertex
fn vs_rect(
    @location(0) corner: vec2<f32>,
    @location(1) min: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) color: vec4<f32>,
) -> VsOut {
    var out: VsOut;
    out.clip = to_clip(min + corner * size);
    out.color = color;
    return out;
}

@vertex
fn vs_belt(@location(0) pos: vec2<f32>, @location(1) color: vec4<f32>) -> VsOut {
    var out: VsOut;
    out.clip = to_clip(pos);
    out.color = color;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return in.color;
}
