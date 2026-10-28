//! The wgpu renderer. A frame has two passes. The first draws the ground, belts, contact
//! shadows, machines, lines and dots into colour, object-ID and depth targets, 4x MSAA where the device
//! allows it. The ground writes no ID, so it is object 0. The second runs the outline pass over
//! every MSAA sample of IDs and depth and resolves into the surface. Model buffers are uploaded
//! once per model set and per scene, never per frame.

use std::mem::size_of;
use std::ops::Range;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::camera::{Camera, NEAR_FRACTION, ORTHO_FAR, Projection};
use crate::edges;
use crate::geometry::{
    self, DETAILS, DotVertex, HaloVertex, InstanceRaw, LineVertex, MeshVertex, RibbonVertex,
    Strokes,
};
use crate::model::{Line3, Mesh, Model};
use crate::scene::{Lod, Scene};
use crate::theme::{self, PALETTE_LEN, TINT_LEN};

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
/// Object IDs (three 8-bit channels) with the surface's fog in alpha.
const ID_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const MSAA: u32 = 4;
/// Thinnest stroke on screen, in device pixels. Thinner strokes fade out instead.
const MIN_STROKE_PX: f32 = 0.75;
/// The depth cleared before a frame. The depth is reverse, so this is the farthest value.
const FAR_DEPTH: f32 = 0.0;

/// Why a renderer could not be created or a frame could not be drawn.
#[non_exhaustive]
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
}

/// The per-frame camera, as the shader reads it. Every field is a `vec4` so the layout needs no
/// padding. See `plant.wgsl` for what each lane means.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable, PartialEq)]
pub struct CameraUniform {
    pub right: [f32; 4],
    pub up: [f32; 4],
    pub toward: [f32; 4],
    pub eye: [f32; 4],
    pub pivot: [f32; 4],
    pub view: [f32; 4],
    pub depth: [f32; 4],
    pub fog: [f32; 4],
}

/// The shader source for a scene target with `samples` MSAA samples per pixel.
pub fn plant_source(samples: u32) -> String {
    let multisampled = if samples > 1 { "multisampled_" } else { "" };
    include_str!("plant.wgsl")
        .replace("@@MS@@", multisampled)
        .replace("@@SAMPLES@@", &samples.to_string())
        .replace("@@PALETTE_LEN@@", &PALETTE_LEN.to_string())
        .replace("@@THEME@@", &theme::wgsl_consts())
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ThemeUniform {
    palette: [[f32; 4]; PALETTE_LEN],
    tints: [[f32; 4]; TINT_LEN],
}

struct MeshBuffers {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    index_count: u32,
}

struct StrokeBuffers {
    buffer: wgpu::Buffer,
    /// Cumulative vertex counts by level of detail.
    counts: [u32; 3],
}

struct ModelBuffers {
    levels: [MeshBuffers; 3],
    lines: StrokeBuffers,
    dots: StrokeBuffers,
}

struct Pipelines {
    grid: wgpu::RenderPipeline,
    ribbon: wgpu::RenderPipeline,
    mesh: wgpu::RenderPipeline,
    halo: wgpu::RenderPipeline,
    line: wgpu::RenderPipeline,
    dot: wgpu::RenderPipeline,
    edge: wgpu::RenderPipeline,
}

/// Targets that the scene pass writes and the outline pass reads.
struct Targets {
    /// The multisampled colour target, or `None` when the surface is drawn directly.
    colour: Option<wgpu::TextureView>,
    id: wgpu::TextureView,
    depth: wgpu::TextureView,
}

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    adapter_info: wgpu::AdapterInfo,
    samples: u32,
    camera_buf: wgpu::Buffer,
    uniform_bind: wgpu::BindGroup,
    edge_layout: wgpu::BindGroupLayout,
    edge_bind: wgpu::BindGroup,
    targets: Targets,
    pipelines: Pipelines,
    clear: wgpu::Color,
    models: Vec<ModelBuffers>,
    /// Six halo vertices per kind, kind by kind.
    halos: Option<wgpu::Buffer>,
    ribbons: Option<(wgpu::Buffer, u32)>,
    instances: Option<wgpu::Buffer>,
    /// Per kind: its range inside `instances`.
    ranges: Vec<Range<u32>>,
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

        let samples = sample_count(&adapter, format);
        let srgb = format.is_srgb();
        let page = theme::page(srgb);
        let theme_uniform = ThemeUniform {
            palette: theme::palette(srgb),
            tints: theme::tints(),
        };
        let camera_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera"),
            size: size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let theme_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("theme"),
            contents: bytemuck::bytes_of(&theme_uniform),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniforms"),
            entries: &[uniform_entry(0), uniform_entry(1)],
        });
        let uniform_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniforms"),
            layout: &uniform_layout,
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
        let edge_layout = edge_layout(&device, samples);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("plant"),
            source: wgpu::ShaderSource::Wgsl(plant_source(samples).into()),
        });
        let pipelines = Pipelines::new(
            &device,
            &shader,
            &uniform_layout,
            &edge_layout,
            format,
            samples,
        );

        let targets = targets(&device, format, samples, config.width, config.height);
        let edge_bind = edge_bind(&device, &edge_layout, &targets);
        Ok(Self {
            adapter_info: adapter.get_info(),
            device,
            queue,
            surface,
            config,
            samples,
            camera_buf,
            uniform_bind,
            edge_layout,
            edge_bind,
            targets,
            pipelines,
            clear: wgpu::Color {
                r: page[0] as f64,
                g: page[1] as f64,
                b: page[2] as f64,
                a: page[3] as f64,
            },
            models: Vec::new(),
            halos: None,
            ribbons: None,
            instances: None,
            ranges: Vec::new(),
        })
    }

    /// The adapter in use, for reporting which backend drew the frames.
    pub fn adapter_info(&self) -> &wgpu::AdapterInfo {
        &self.adapter_info
    }

    /// The format of the surface the frames are presented to.
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    /// The MSAA sample count of the scene pass: 4, or 1 where the device lacks it.
    pub fn samples(&self) -> u32 {
        self.samples
    }

    /// Resizes the surface and its targets. `size` is the drawable size in physical pixels.
    pub fn resize(&mut self, size: [u32; 2]) {
        self.config.width = size[0].max(1);
        self.config.height = size[1].max(1);
        self.surface.configure(&self.device, &self.config);
        self.targets = targets(
            &self.device,
            self.config.format,
            self.samples,
            self.config.width,
            self.config.height,
        );
        self.edge_bind = edge_bind(&self.device, &self.edge_layout, &self.targets);
    }

    /// Uploads every kind's three meshes, lines, dots and halo once. Instances refer to kinds by
    /// index into this slice. Call `set_scene` afterwards, because the instance ranges depend
    /// on the kinds. Each level's feature edges are extracted here, from its mesh, and drawn
    /// at that level's detail.
    pub fn set_models(&mut self, models: &[Model]) {
        self.models = models
            .iter()
            .map(|model| {
                let mut lines: Vec<Line3> = model.lines.clone();
                for (mesh, detail) in model.meshes.iter().zip(DETAILS) {
                    lines.extend(edges::feature_lines(mesh, detail));
                }
                ModelBuffers {
                    levels: model.meshes.each_ref().map(|mesh| self.mesh_buffers(mesh)),
                    lines: self.strokes(
                        "lines",
                        geometry::line_vertices(&lines),
                        wgpu::BufferUsages::VERTEX,
                    ),
                    dots: self.strokes(
                        "dots",
                        geometry::dot_vertices(&model.dots),
                        wgpu::BufferUsages::VERTEX,
                    ),
                }
            })
            .collect();
        let halos: Vec<HaloVertex> = models
            .iter()
            .flat_map(|model| geometry::halo_vertices(&model.bounds))
            .collect();
        self.halos = Some(upload(
            &self.device,
            "halos",
            wgpu::BufferUsages::VERTEX,
            &halos,
        ));
        self.ranges.clear();
        self.instances = None;
    }

    /// Uploads the belts and the machine instances. Call when the plant changes, not per frame.
    ///
    /// # Panics
    ///
    /// If an instance names a kind that `set_models` did not upload.
    pub fn set_scene(&mut self, scene: &Scene) {
        let ribbons = geometry::ribbon_vertices(&scene.ribbons);
        self.ribbons = Some((
            upload(
                &self.device,
                "ribbons",
                wgpu::BufferUsages::VERTEX,
                &ribbons,
            ),
            ribbons.len() as u32,
        ));
        let (raw, ranges) = geometry::instance_data(&scene.instances, self.models.len());
        self.instances = Some(upload(
            &self.device,
            "instances",
            wgpu::BufferUsages::VERTEX,
            &raw,
        ));
        self.ranges = ranges;
    }

    /// Draws one frame at the given level of detail. Returns `Ok(false)` for a frame the surface
    /// skipped (timeout, occlusion, or a resize in flight).
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
            bytemuck::bytes_of(&self.uniform(camera)),
        );
        let surface_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });
        let scene_colour = self.targets.colour.as_ref().unwrap_or(&surface_view);
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("scene"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: scene_colour,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(self.clear),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.targets.id,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.targets.depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(FAR_DEPTH),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.draw_scene(&mut pass, lod);
        }
        {
            let (view, resolve) = match &self.targets.colour {
                Some(msaa) => (msaa, Some(&surface_view)),
                None => (&surface_view, None),
            };
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("edges"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: resolve,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: if resolve.is_some() {
                            wgpu::StoreOp::Discard
                        } else {
                            wgpu::StoreOp::Store
                        },
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipelines.edge);
            pass.set_bind_group(0, &self.uniform_bind, &[]);
            pass.set_bind_group(1, &self.edge_bind, &[]);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit([encoder.finish()]);
        self.queue.present(frame);
        Ok(true)
    }

    fn draw_scene(&self, pass: &mut wgpu::RenderPass<'_>, lod: Lod) {
        pass.set_bind_group(0, &self.uniform_bind, &[]);
        pass.set_pipeline(&self.pipelines.grid);
        pass.draw(0..3, 0..1);

        let Some(instances) = &self.instances else {
            return;
        };
        if let Some((ribbons, count)) = &self.ribbons
            && *count > 0
        {
            pass.set_pipeline(&self.pipelines.ribbon);
            pass.set_vertex_buffer(0, ribbons.slice(..));
            pass.draw(0..*count, 0..1);
        }

        if let Some(halos) = &self.halos {
            pass.set_pipeline(&self.pipelines.halo);
            pass.set_vertex_buffer(0, halos.slice(..));
            pass.set_vertex_buffer(1, instances.slice(..));
            for (kind, range) in self.ranges.iter().enumerate() {
                if !range.is_empty() {
                    let first = 6 * kind as u32;
                    pass.draw(first..first + 6, range.clone());
                }
            }
        }

        pass.set_pipeline(&self.pipelines.mesh);
        for (model, range) in self.models.iter().zip(&self.ranges) {
            if range.is_empty() {
                continue;
            }
            for mesh in &model.levels[..=lod as usize] {
                if mesh.index_count == 0 {
                    continue;
                }
                pass.set_vertex_buffer(0, mesh.vertices.slice(..));
                pass.set_vertex_buffer(1, instances.slice(..));
                pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..mesh.index_count, 0, range.clone());
            }
        }

        pass.set_pipeline(&self.pipelines.line);
        for (model, range) in self.models.iter().zip(&self.ranges) {
            let count = model.lines.counts[lod as usize];
            if count == 0 || range.is_empty() {
                continue;
            }
            pass.set_vertex_buffer(0, model.lines.buffer.slice(..));
            pass.set_vertex_buffer(1, instances.slice(..));
            pass.draw(0..count, range.clone());
        }

        pass.set_pipeline(&self.pipelines.dot);
        for (model, range) in self.models.iter().zip(&self.ranges) {
            let count = model.dots.counts[lod as usize];
            if count == 0 || range.is_empty() {
                continue;
            }
            pass.set_vertex_buffer(0, model.dots.buffer.slice(..));
            pass.set_vertex_buffer(1, instances.slice(..));
            pass.draw(0..count, range.clone());
        }
    }

    fn uniform(&self, camera: &Camera) -> CameraUniform {
        let b = camera.basis();
        let pivot = camera.pivot();
        let eye = camera.eye();
        let [vw, vh] = camera.viewport();
        let d = camera.eye_distance();
        let ortho = match camera.projection() {
            Projection::Perspective => 0.0,
            Projection::Orthographic => 1.0,
        };
        let (fog_near, fog_far) = (theme::FOG_NEAR, theme::FOG_FAR);
        let (angle_lo, angle_hi) = theme::ANGLE_FADE;
        CameraUniform {
            right: [b.right[0], b.right[1], b.right[2], 0.0],
            up: [b.up[0], b.up[1], b.up[2], 0.0],
            toward: [b.toward[0], b.toward[1], b.toward[2], 0.0],
            eye: [eye[0], eye[1], eye[2], ortho],
            pivot: [pivot[0], pivot[1], d, 0.0],
            view: [vw, vh, camera.zoom(), MIN_STROKE_PX],
            depth: [NEAR_FRACTION * d, ORTHO_FAR * d, camera.focal(), 0.0],
            fog: [fog_near, fog_far, angle_lo, angle_hi],
        }
    }

    fn mesh_buffers(&self, mesh: &Mesh) -> MeshBuffers {
        let vertices = geometry::mesh_vertices(mesh);
        MeshBuffers {
            vertices: upload(
                &self.device,
                "mesh vertices",
                wgpu::BufferUsages::VERTEX,
                &vertices,
            ),
            indices: upload(
                &self.device,
                "mesh indices",
                wgpu::BufferUsages::INDEX,
                &mesh.indices,
            ),
            index_count: mesh.indices.len() as u32,
        }
    }

    fn strokes<V: Pod>(
        &self,
        label: &str,
        strokes: Strokes<V>,
        usage: wgpu::BufferUsages,
    ) -> StrokeBuffers {
        StrokeBuffers {
            buffer: upload(&self.device, label, usage, &strokes.vertices),
            counts: strokes.counts,
        }
    }
}

/// 4x when the surface format and both targets support it with resolve, otherwise 1.
fn sample_count(adapter: &wgpu::Adapter, surface: wgpu::TextureFormat) -> u32 {
    let supports = |format: wgpu::TextureFormat| {
        adapter
            .get_texture_format_features(format)
            .flags
            .sample_count_supported(MSAA)
    };
    let resolves = adapter
        .get_texture_format_features(surface)
        .flags
        .contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_RESOLVE);
    if resolves && supports(surface) && supports(ID_FORMAT) && supports(DEPTH_FORMAT) {
        MSAA
    } else {
        1
    }
}

fn targets(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    samples: u32,
    width: u32,
    height: u32,
) -> Targets {
    let make = |label: &str, format: wgpu::TextureFormat, usage: wgpu::TextureUsages| {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: width.max(1),
                    height: height.max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: samples,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default())
    };
    let sampled = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
    Targets {
        colour: (samples > 1).then(|| {
            make(
                "scene colour",
                format,
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            )
        }),
        id: make("scene ids", ID_FORMAT, sampled),
        depth: make("scene depth", DEPTH_FORMAT, sampled),
    }
}

fn edge_layout(device: &wgpu::Device, samples: u32) -> wgpu::BindGroupLayout {
    let multisampled = samples > 1;
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("edge inputs"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled,
                },
                count: None,
            },
        ],
    })
}

fn edge_bind(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    targets: &Targets,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("edge inputs"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&targets.depth),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&targets.id),
            },
        ],
    })
}

impl Pipelines {
    fn new(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        uniform_layout: &wgpu::BindGroupLayout,
        edge_layout: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
        samples: u32,
    ) -> Self {
        let scene_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("scene"),
            bind_group_layouts: &[Some(uniform_layout)],
            immediate_size: 0,
        });
        let edge_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("edge"),
            bind_group_layouts: &[Some(uniform_layout), Some(edge_layout)],
            immediate_size: 0,
        });

        let colour = |blend: Option<wgpu::BlendState>| {
            Some(wgpu::ColorTargetState {
                format,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })
        };
        let ids = |write: bool| {
            Some(wgpu::ColorTargetState {
                format: ID_FORMAT,
                blend: None,
                write_mask: if write {
                    wgpu::ColorWrites::ALL
                } else {
                    wgpu::ColorWrites::empty()
                },
            })
        };
        let depth = |write: bool, compare: wgpu::CompareFunction| {
            Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(write),
                depth_compare: Some(compare),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            })
        };
        let multisample = wgpu::MultisampleState {
            count: samples,
            ..Default::default()
        };

        let mesh_vertex = wgpu::VertexBufferLayout {
            array_stride: size_of::<MeshVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Uint32],
        };
        let mesh_instance = wgpu::VertexBufferLayout {
            array_stride: size_of::<InstanceRaw>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![3 => Float32x3, 4 => Uint32, 5 => Uint32],
        };
        let line_vertex = wgpu::VertexBufferLayout {
            array_stride: size_of::<LineVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2, 3 => Float32, 4 => Uint32],
        };
        let dot_vertex = wgpu::VertexBufferLayout {
            array_stride: size_of::<DotVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x3, 3 => Float32x2, 4 => Uint32],
        };
        let ribbon_vertex = wgpu::VertexBufferLayout {
            array_stride: size_of::<RibbonVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32, 2 => Float32, 3 => Float32, 4 => Float32],
        };
        let halo_vertex = wgpu::VertexBufferLayout {
            array_stride: size_of::<HaloVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4],
        };
        let offset_instance = wgpu::VertexBufferLayout {
            array_stride: size_of::<InstanceRaw>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![5 => Float32x3],
        };
        let halo_instance = wgpu::VertexBufferLayout {
            array_stride: size_of::<InstanceRaw>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![2 => Float32x3],
        };

        let make = |label: &str,
                    layout: &wgpu::PipelineLayout,
                    vs: &str,
                    fs: &str,
                    buffers: &[Option<wgpu::VertexBufferLayout>],
                    depth_state: Option<wgpu::DepthStencilState>,
                    targets: &[Option<wgpu::ColorTargetState>]|
         -> wgpu::RenderPipeline {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module: shader,
                    entry_point: Some(vs),
                    compilation_options: Default::default(),
                    buffers,
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: depth_state,
                multisample,
                fragment: Some(wgpu::FragmentState {
                    module: shader,
                    entry_point: Some(fs),
                    compilation_options: Default::default(),
                    targets,
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let alpha = Some(wgpu::BlendState::ALPHA_BLENDING);
        // Depth is reverse, so a nearer surface has the larger value.
        let nearer = wgpu::CompareFunction::GreaterEqual;
        Self {
            grid: make(
                "grid",
                &scene_layout,
                "vs_full",
                "fs_grid",
                &[],
                depth(true, wgpu::CompareFunction::Always),
                &[colour(None), ids(false)],
            ),
            // Belts and shadows lie on the ground, which is drawn first and lies under them, so
            // they skip the depth test. Testing them against the ground's per-pixel depth lets
            // the grid through at the far samples of a pixel.
            ribbon: make(
                "ribbons",
                &scene_layout,
                "vs_ribbon",
                "fs_ribbon",
                &[Some(ribbon_vertex)],
                depth(true, wgpu::CompareFunction::Always),
                &[colour(None), ids(false)],
            ),
            mesh: make(
                "mesh",
                &scene_layout,
                "vs_mesh",
                "fs_mesh",
                &[Some(mesh_vertex), Some(mesh_instance)],
                depth(true, nearer),
                &[colour(None), ids(true)],
            ),
            halo: make(
                "halo",
                &scene_layout,
                "vs_halo",
                "fs_halo",
                &[Some(halo_vertex), Some(halo_instance)],
                depth(false, wgpu::CompareFunction::Always),
                &[colour(alpha), ids(false)],
            ),
            line: make(
                "lines",
                &scene_layout,
                "vs_line",
                "fs_line",
                &[Some(line_vertex), Some(offset_instance.clone())],
                depth(false, nearer),
                &[colour(alpha), ids(false)],
            ),
            dot: make(
                "dots",
                &scene_layout,
                "vs_dot",
                "fs_dot",
                &[Some(dot_vertex), Some(offset_instance)],
                depth(false, nearer),
                &[colour(alpha), ids(false)],
            ),
            edge: make(
                "edges",
                &edge_pipeline_layout,
                "vs_full",
                "fs_edge",
                &[],
                None,
                &[colour(alpha)],
            ),
        }
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
