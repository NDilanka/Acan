use std::mem::size_of;

use wgpu::util::DeviceExt;

use crate::camera::{Camera, CameraUniform};
use crate::scene::{BeltVertex, Rect, Scene, tessellate_polyline};

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("no compatible GPU adapter: {0}")]
    Adapter(#[from] wgpu::RequestAdapterError),
    #[error("device request failed: {0}")]
    Device(#[from] wgpu::RequestDeviceError),
    #[error("the adapter cannot present to this surface")]
    UnsupportedSurface,
    #[error("surface lost; recreate the renderer")]
    SurfaceLost,
}

const UNIT_QUAD: [[f32; 2]; 6] = [
    [0.0, 0.0],
    [1.0, 0.0],
    [1.0, 1.0],
    [0.0, 0.0],
    [1.0, 1.0],
    [0.0, 1.0],
];

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    adapter_info: wgpu::AdapterInfo,
    camera_buf: wgpu::Buffer,
    camera_bind: wgpu::BindGroup,
    rect_pipeline: wgpu::RenderPipeline,
    belt_pipeline: wgpu::RenderPipeline,
    quad_buf: wgpu::Buffer,
    rects: Option<(wgpu::Buffer, u32)>,
    belts: Option<(wgpu::Buffer, u32)>,
    clear: [f32; 4],
}

impl Renderer {
    /// `size` is the drawable size in physical pixels.
    pub async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        size: [u32; 2],
    ) -> Result<Self, RenderError> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("acan-render"),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                ..Default::default()
            })
            .await?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .or_else(|| caps.formats.first().copied())
            .ok_or(RenderError::UnsupportedSurface)?;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: size[0].max(1),
            height: size[1].max(1),
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::include_wgsl!("plant.wgsl"));
        let camera_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera"),
            size: size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let camera_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera"),
            layout: &bind_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: camera_buf.as_entire_binding() }],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("plant"),
            bind_group_layouts: &[Some(&bind_layout)],
            immediate_size: 0,
        });

        let quad_layout = wgpu::VertexBufferLayout {
            array_stride: size_of::<[f32; 2]>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x2],
        };
        let rect_layout = wgpu::VertexBufferLayout {
            array_stride: size_of::<Rect>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![1 => Float32x2, 2 => Float32x2, 3 => Float32x4],
        };
        let belt_layout = wgpu::VertexBufferLayout {
            array_stride: size_of::<BeltVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4],
        };
        let pipeline = |label, entry, buffers: &[Option<wgpu::VertexBufferLayout>]| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    buffers,
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let rect_pipeline = pipeline("rects", "vs_rect", &[Some(quad_layout), Some(rect_layout)]);
        let belt_pipeline = pipeline("belts", "vs_belt", &[Some(belt_layout)]);

        let quad_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("unit quad"),
            contents: bytemuck::cast_slice(&UNIT_QUAD),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Ok(Self {
            adapter_info: adapter.get_info(),
            device,
            queue,
            surface,
            config,
            camera_buf,
            camera_bind,
            rect_pipeline,
            belt_pipeline,
            quad_buf,
            rects: None,
            belts: None,
            clear: [0.11, 0.115, 0.12, 1.0],
        })
    }

    pub fn adapter_info(&self) -> &wgpu::AdapterInfo {
        &self.adapter_info
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    /// Background color, sRGB.
    pub fn set_clear_color(&mut self, color: [f32; 4]) {
        self.clear = color;
    }

    pub fn resize(&mut self, size: [u32; 2]) {
        self.config.width = size[0].max(1);
        self.config.height = size[1].max(1);
        self.surface.configure(&self.device, &self.config);
    }

    /// Uploads the scene geometry. Call when the scene changes, not per frame.
    pub fn set_scene(&mut self, scene: &Scene) {
        let srgb = self.config.format.is_srgb();
        let rects: Vec<Rect> = scene
            .rects
            .iter()
            .map(|r| Rect { color: self.target_color(r.color, srgb), ..*r })
            .collect();
        let mut belts = Vec::new();
        for line in &scene.belts {
            tessellate_polyline(line, &mut belts);
        }
        for v in &mut belts {
            v.color = self.target_color(v.color, srgb);
        }
        self.rects = self.upload("rects", &rects);
        self.belts = self.upload("belts", &belts);
    }

    fn target_color(&self, c: [f32; 4], srgb: bool) -> [f32; 4] {
        if srgb { srgb_to_linear(c) } else { c }
    }

    fn upload<T: bytemuck::Pod>(&self, label: &str, data: &[T]) -> Option<(wgpu::Buffer, u32)> {
        if data.is_empty() {
            return None;
        }
        let buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(data),
            usage: wgpu::BufferUsages::VERTEX,
        });
        Some((buf, data.len() as u32))
    }

    /// Draws one frame. Skipped frames (timeout, occlusion) return `Ok(false)`.
    pub fn render(&mut self, camera: &Camera) -> Result<bool, RenderError> {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(f) => f,
            wgpu::CurrentSurfaceTexture::Suboptimal(f) => {
                self.surface.configure(&self.device, &self.config);
                f
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(false);
            }
            wgpu::CurrentSurfaceTexture::Lost => return Err(RenderError::SurfaceLost),
            _ => return Ok(false),
        };
        self.queue.write_buffer(&self.camera_buf, 0, bytemuck::bytes_of(&camera.uniform()));
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let clear = self.target_color(self.clear, self.config.format.is_srgb());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("plant"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: clear[0] as f64,
                            g: clear[1] as f64,
                            b: clear[2] as f64,
                            a: clear[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_bind_group(0, &self.camera_bind, &[]);
            if let Some((buf, count)) = &self.belts {
                pass.set_pipeline(&self.belt_pipeline);
                pass.set_vertex_buffer(0, buf.slice(..));
                pass.draw(0..*count, 0..1);
            }
            if let Some((buf, count)) = &self.rects {
                pass.set_pipeline(&self.rect_pipeline);
                pass.set_vertex_buffer(0, self.quad_buf.slice(..));
                pass.set_vertex_buffer(1, buf.slice(..));
                pass.draw(0..UNIT_QUAD.len() as u32, 0..*count);
            }
        }
        self.queue.submit([encoder.finish()]);
        self.queue.present(frame);
        Ok(true)
    }
}

fn srgb_to_linear(c: [f32; 4]) -> [f32; 4] {
    let f = |x: f32| if x <= 0.04045 { x / 12.92 } else { ((x + 0.055) / 1.055).powf(2.4) };
    [f(c[0]), f(c[1]), f(c[2]), c[3]]
}
