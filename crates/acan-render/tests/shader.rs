//! The shader must parse and validate in both sample-count variants. wgpu reports a bad shader
//! through its error handler, which a browser page can hide, so this check runs on the host.

use acan_render::renderer::plant_source;

#[test]
fn plant_shader_validates_single_and_multisampled() {
    for (label, samples) in [("single", 1), ("multisampled", 4)] {
        let wgsl = plant_source(samples);
        let module = naga::front::wgsl::parse_str(&wgsl)
            .unwrap_or_else(|error| panic!("{label}: {}", error.emit_to_string(&wgsl)));
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::default(),
        )
        .validate(&module)
        .unwrap_or_else(|error| panic!("{label}: {error:?}"));
    }
}
