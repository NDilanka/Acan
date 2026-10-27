use std::mem::size_of;
use std::ops::Range;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::camera::Camera;
use crate::draw_list::DrawList;
use crate::geometry::{
    Batch, Class, DotVertex, FillVertex, KindSpans, LineVertex, TessellationError,
};
use crate::scene::{Instance, Lod, Scene};
use crate::theme::{self, PALETTE_LEN, TINT_LEN};

const SAMPLES: u32 = 4;
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
/// Thinnest stroke on screen, in device pixels. Thinner strokes fade out instead.
const MIN_PX: f32 = 0.75;
/// Width of the floor's soft halo, in plan units.
const HALO_WIDTH: f32 = 28.0;

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
    #[error("validation error while acquiring the surface texture")]
    SurfaceValidation,
    #[error("tessellation failed: {0}")]
    Tessellation(#[from] TessellationError),
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ThemeUniform {
    palette: [[f32; 4]; PALETTE_LEN],
    tints: [[f32; 4]; TINT_LEN],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InstanceRaw {
    offset: [f32; 2],
    rank: f32,
    tint: u32,
}

/// One set of GPU buffers for a batch of kinds.
struct Gpu {
    fill: wgpu::Buffer,
    fill_opaque: wgpu::Buffer,
    fill_halo: wgpu::Buffer,
    line: wgpu::Buffer,
    line_idx: wgpu::Buffer,
    dot: wgpu::Buffer,
    dot_idx: wgpu::Buffer,
    kinds: Vec<KindSpans>,
}

impl Gpu {
    fn new(device: &wgpu::Device, batch: &Batch) -> Self {
        let vb = wgpu::BufferUsages::VERTEX;
        let ib = wgpu::BufferUsages::INDEX;
        Self {
            fill: upload(device, "fill vertices", vb, &batch.fill),
            fill_opaque: upload(device, "fill opaque indices", ib, &batch.fill_opaque),
            fill_halo: upload(device, "fill halo indices", ib, &batch.fill_halo),
            line: upload(device, "line vertices", vb, &batch.line),
            line_idx: upload(device, "line indices", ib, &batch.line_idx),
            dot: upload(device, "dot vertices", vb, &batch.dot),
            dot_idx: upload(device, "dot indices", ib, &batch.dot_idx),
            kinds: batch.kinds.clone(),
        }
    }
}

struct Pipelines {
    fill_opaque: wgpu::RenderPipeline,
    fill_halo: wgpu::RenderPipeline,
    line: wgpu::RenderPipeline,
    dot: wgpu::RenderPipeline,
}

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    adapter_info: wgpu::AdapterInfo,
    camera_buf: wgpu::Buffer,
    bind: wgpu::BindGroup,
    pipelines: Pipelines,
    msaa_color: wgpu::TextureView,
    depth: wgpu::TextureView,
    clear: wgpu::Color,
    models: Option<Gpu>,
    world: Option<Gpu>,
    instances: Option<wgpu::Buffer>,
    /// Per model kind: its range inside `instances`.
    instance_ranges: Vec<Range<u32>>,
    depth_step: f32,
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
            alpha_mode: caps
                .alpha_modes
                .first()
                .copied()
                .ok_or(RenderError::UnsupportedSurface)?,
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let srgb = format.is_srgb();
        let page = theme::page(srgb);
        let theme_uniform = ThemeUniform {
            palette: theme::palette(srgb),
            tints: theme::tints(),
        };
        let camera_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera"),
            size: size_of::<crate::camera::CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let theme_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("theme"),
            contents: bytemuck::bytes_of(&theme_uniform),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("plant"),
            entries: &[uniform_entry(0), uniform_entry(1)],
        });
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("plant"),
            layout: &bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: theme_buf.as_entire_binding(),
                },
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("plant"),
            bind_group_layouts: &[Some(&bind_layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::include_wgsl!("plant.wgsl"));

        let fill_vertex = wgpu::VertexBufferLayout {
            array_stride: size_of::<FillVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32, 2 => Uint32, 3 => Float32],
        };
        let fill_instance = wgpu::VertexBufferLayout {
            array_stride: size_of::<InstanceRaw>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![4 => Float32x2, 5 => Float32, 6 => Uint32],
        };
        let line_vertex = wgpu::VertexBufferLayout {
            array_stride: size_of::<LineVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x2, 3 => Float32, 4 => Uint32, 5 => Float32],
        };
        let line_instance = wgpu::VertexBufferLayout {
            array_stride: size_of::<InstanceRaw>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![6 => Float32x2, 7 => Float32, 8 => Uint32],
        };
        let dot_vertex = wgpu::VertexBufferLayout {
            array_stride: size_of::<DotVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32, 3 => Uint32, 4 => Float32],
        };
        let dot_instance = wgpu::VertexBufferLayout {
            array_stride: size_of::<InstanceRaw>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![5 => Float32x2, 6 => Float32, 7 => Uint32],
        };
        let make = |label: &str,
                    vs: &str,
                    fs: &str,
                    buffers: &[Option<wgpu::VertexBufferLayout>],
                    depth_write: bool| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some(vs),
                    compilation_options: Default::default(),
                    buffers,
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(depth_write),
                    depth_compare: Some(wgpu::CompareFunction::LessEqual),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: SAMPLES,
                    ..Default::default()
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(fs),
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
        let pipelines = Pipelines {
            fill_opaque: make(
                "fill opaque",
                "vs_fill",
                "fs_fill",
                &[Some(fill_vertex.clone()), Some(fill_instance.clone())],
                true,
            ),
            fill_halo: make(
                "fill halo",
                "vs_fill",
                "fs_fill",
                &[Some(fill_vertex), Some(fill_instance)],
                false,
            ),
            line: make(
                "lines",
                "vs_line",
                "fs_line",
                &[Some(line_vertex), Some(line_instance)],
                true,
            ),
            dot: make(
                "dots",
                "vs_dot",
                "fs_dot",
                &[Some(dot_vertex), Some(dot_instance)],
                true,
            ),
        };
        let (msaa_color, depth) = targets(&device, format, config.width, config.height);

        Ok(Self {
            adapter_info: adapter.get_info(),
            device,
            queue,
            surface,
            config,
            camera_buf,
            bind,
            pipelines,
            msaa_color,
            depth,
            clear: wgpu::Color {
                r: page[0] as f64,
                g: page[1] as f64,
                b: page[2] as f64,
                a: page[3] as f64,
            },
            models: None,
            world: None,
            instances: None,
            instance_ranges: Vec::new(),
            depth_step: 1.0,
        })
    }

    pub fn adapter_info(&self) -> &wgpu::AdapterInfo {
        &self.adapter_info
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    pub fn resize(&mut self, size: [u32; 2]) {
        self.config.width = size[0].max(1);
        self.config.height = size[1].max(1);
        self.surface.configure(&self.device, &self.config);
        let (msaa, depth) = targets(
            &self.device,
            self.config.format,
            self.config.width,
            self.config.height,
        );
        self.msaa_color = msaa;
        self.depth = depth;
    }

    /// Uploads and tessellates the model kinds once. Instances in a scene refer to these by index.
    /// Call `set_scene` after this, because it clears the scene.
    pub fn set_models(&mut self, models: &[DrawList]) -> Result<(), RenderError> {
        let mut batch = Batch::default();
        for model in models {
            batch.push_list(model)?;
        }
        self.models = Some(Gpu::new(&self.device, &batch));
        self.instance_ranges = vec![0..0; models.len()];
        self.instances = None;
        Ok(())
    }

    /// Uploads the ground and the machine instances. Call when the plant changes, not per frame.
    pub fn set_scene(&mut self, scene: &Scene) -> Result<(), RenderError> {
        let mut world = Batch::default();
        world.push_list(&scene.world)?;
        if let Some((min, max)) = scene.floor_halo {
            world.push_floor_halo(min, max, HALO_WIDTH);
        }
        self.world = Some(Gpu::new(&self.device, &world));

        let mut raw = Vec::with_capacity(scene.instances.len() + 1);
        raw.push(InstanceRaw {
            offset: [0.0, 0.0],
            rank: 0.0,
            tint: 0,
        });
        for (kind, range) in self.instance_ranges.iter_mut().enumerate() {
            let start = raw.len() as u32;
            for (i, inst) in scene.instances.iter().enumerate() {
                if usize::from(inst.kind) == kind {
                    raw.push(instance_raw(inst, i));
                }
            }
            *range = start..raw.len() as u32;
        }
        self.instances = Some(upload(
            &self.device,
            "instances",
            wgpu::BufferUsages::VERTEX,
            &raw,
        ));
        self.depth_step = 1.0 / (scene.instances.len() as f32 + 2.0);
        Ok(())
    }

    /// Draws one frame. Skipped frames (timeout, occlusion) return `Ok(false)`.
    pub fn render(&mut self, camera: &Camera, lod: Lod) -> Result<bool, RenderError> {
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
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(RenderError::SurfaceValidation);
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(false);
            }
        };
        self.queue.write_buffer(
            &self.camera_buf,
            0,
            bytemuck::bytes_of(&camera.uniform(self.depth_step, MIN_PX)),
        );
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("plant"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.msaa_color,
                    depth_slice: None,
                    resolve_target: Some(&view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear),
                        store: wgpu::StoreOp::Discard,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if let Some(instances) = &self.instances {
                pass.set_bind_group(0, &self.bind, &[]);
                pass.set_vertex_buffer(1, instances.slice(..));
                pass.set_pipeline(&self.pipelines.fill_opaque);
                self.draw_class(&mut pass, Class::Opaque, lod);
                pass.set_pipeline(&self.pipelines.line);
                self.draw_class(&mut pass, Class::Line, lod);
                pass.set_pipeline(&self.pipelines.dot);
                self.draw_class(&mut pass, Class::Dot, lod);
                pass.set_pipeline(&self.pipelines.fill_halo);
                self.draw_class(&mut pass, Class::Halo, lod);
            }
        }
        self.queue.submit([encoder.finish()]);
        self.queue.present(frame);
        Ok(true)
    }

    fn draw_class(&self, pass: &mut wgpu::RenderPass<'_>, class: Class, lod: Lod) {
        for (is_world, gpu) in [(true, &self.world), (false, &self.models)] {
            let Some(gpu) = gpu else { continue };
            let (vb, ib) = match class {
                Class::Opaque => (&gpu.fill, &gpu.fill_opaque),
                Class::Halo => (&gpu.fill, &gpu.fill_halo),
                Class::Line => (&gpu.line, &gpu.line_idx),
                Class::Dot => (&gpu.dot, &gpu.dot_idx),
            };
            pass.set_vertex_buffer(0, vb.slice(..));
            pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
            for (kind, spans) in gpu.kinds.iter().enumerate() {
                let instances = if is_world {
                    0..1
                } else {
                    match self.instance_ranges.get(kind) {
                        Some(range) => range.clone(),
                        None => continue,
                    }
                };
                if instances.is_empty() {
                    continue;
                }
                for span in &spans[class as usize][lod as usize] {
                    pass.draw_indexed(span.start..span.end, 0, instances.clone());
                }
            }
        }
    }
}

fn instance_raw(inst: &Instance, index: usize) -> InstanceRaw {
    InstanceRaw {
        offset: inst.offset,
        rank: (index + 1) as f32,
        tint: u32::from(inst.tint),
    }
}

fn uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn targets(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
) -> (wgpu::TextureView, wgpu::TextureView) {
    let make = |label: &str, format: wgpu::TextureFormat| {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: width.max(1),
                    height: height.max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: SAMPLES,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default())
    };
    (make("msaa color", format), make("depth", DEPTH_FORMAT))
}

const EMPTY: [u8; 16] = [0; 16];

/// Uploads `data`. An empty list gets a placeholder so the buffer is never zero sized.
fn upload<T: Pod>(
    device: &wgpu::Device,
    label: &str,
    usage: wgpu::BufferUsages,
    data: &[T],
) -> wgpu::Buffer {
    let bytes = bytemuck::cast_slice::<T, u8>(data);
    let contents = if bytes.is_empty() { &EMPTY[..] } else { bytes };
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents,
        usage,
    })
}
