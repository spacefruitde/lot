//! Renderer der 3D-Ansicht auf Basis von wgpu.
//!
//! Läuft auf dem Mac über Metal und zeichnet in den CAMetalLayer eines NSView
//! der App ([`Renderer::for_appkit_view`]). Ohne Fenster zeichnet er in eine
//! Textur und liefert ein Bild ([`Renderer::headless`]), etwa für Prüfpakete
//! und Tests. Plattformabhängig ist nur, welches Handle wgpu bekommt; der
//! Code hier baut auch unter Linux.

mod camera;

use std::ffi::c_void;
use std::ptr::NonNull;

use bytemuck::{Pod, Zeroable};
pub use camera::OrbitCamera;
use glam::Vec3;
use lot_geom::ModelGeometry;
use wgpu::util::DeviceExt;

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("Keine passende Grafikkarte gefunden: {0}")]
    NoAdapter(String),
    #[error("Grafikkarte nicht nutzbar: {0}")]
    Device(String),
    #[error("Zeichenfläche nicht nutzbar: {0}")]
    Surface(String),
}

const SAMPLES: u32 = 4;
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
/// Hintergrund, Bauteile und Kanten in sRGB.
const BACKGROUND: [f64; 3] = [0.94, 0.94, 0.93];
const BASE: [f32; 3] = [0.93, 0.92, 0.89];
const EDGE: [f32; 3] = [0.22, 0.22, 0.24];

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Globals {
    view_proj: [[f32; 4]; 4],
    light: [f32; 4],
    base: [f32; 4],
    edge: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct MeshVertex {
    position: [f32; 3],
    normal: [f32; 3],
}

enum Target {
    Surface { surface: wgpu::Surface<'static>, config: wgpu::SurfaceConfiguration },
    Offscreen { texture: wgpu::Texture },
}

struct Buffers {
    mesh: wgpu::Buffer,
    mesh_count: u32,
    edges: wgpu::Buffer,
    edge_count: u32,
}

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    target: Target,
    format: wgpu::TextureFormat,
    size: (u32, u32),
    scale: f32,
    msaa: wgpu::TextureView,
    depth: wgpu::TextureView,
    globals: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    mesh_pipeline: wgpu::RenderPipeline,
    edge_pipeline: wgpu::RenderPipeline,
    buffers: Option<Buffers>,
    bounds: Option<(Vec3, Vec3)>,
    triangles: u32,
    camera: OrbitCamera,
}

impl Renderer {
    /// Renderer für einen NSView der Mac-App.
    ///
    /// # Safety
    ///
    /// `ns_view` zeigt auf einen gültigen NSView, der länger lebt als der
    /// Renderer. Aufrufe kommen vom Main-Thread.
    pub unsafe fn for_appkit_view(
        ns_view: NonNull<c_void>,
        width: u32,
        height: u32,
        scale: f32,
    ) -> Result<Self, RenderError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let handle = raw_window_handle::AppKitWindowHandle::new(ns_view);
        let target = wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: Some(raw_window_handle::RawDisplayHandle::AppKit(
                raw_window_handle::AppKitDisplayHandle::new(),
            )),
            raw_window_handle: raw_window_handle::RawWindowHandle::AppKit(handle),
        };
        // SAFETY: Der Aufrufer garantiert, dass der View gültig ist und den Renderer überlebt.
        let surface = unsafe { instance.create_surface_unsafe(target) }
            .map_err(|e| RenderError::Surface(e.to_string()))?;
        let (adapter, device, queue) = open_device(&instance, Some(&surface))?;
        let mut config = surface
            .get_default_config(&adapter, width.max(1), height.max(1))
            .ok_or_else(|| RenderError::Surface("Grafikkarte kann nicht in diese Fläche zeichnen".into()))?;
        let caps = surface.get_capabilities(&adapter);
        config.format = caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(config.format);
        // Fifo wartet auf den Bildschirmtakt und läuft auch mit 120 Hz.
        config.present_mode = wgpu::PresentMode::Fifo;
        surface.configure(&device, &config);
        let format = config.format;
        Ok(Self::build(device, queue, Target::Surface { surface, config }, format, (width, height), scale))
    }

    /// Renderer ohne Fenster, der in eine Textur zeichnet.
    pub fn headless(width: u32, height: u32) -> Result<Self, RenderError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let (_, device, queue) = open_device(&instance, None)?;
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen"),
            size: extent(width, height),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        Ok(Self::build(device, queue, Target::Offscreen { texture }, format, (width, height), 1.0))
    }

    fn build(
        device: wgpu::Device,
        queue: wgpu::Queue,
        target: Target,
        format: wgpu::TextureFormat,
        size: (u32, u32),
        scale: f32,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let globals = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("globals"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("globals"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: globals.as_entire_binding() }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("3d"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });

        let mesh_attributes = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];
        let edge_attributes = wgpu::vertex_attr_array![0 => Float32x3];
        let pipeline = |label: &str,
                        vs: &str,
                        fs: &str,
                        topology: wgpu::PrimitiveTopology,
                        layout: wgpu::VertexBufferLayout<'_>,
                        bias: wgpu::DepthBiasState| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some(vs),
                    compilation_options: Default::default(),
                    buffers: &[Some(layout)],
                },
                primitive: wgpu::PrimitiveState { topology, ..Default::default() },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::LessEqual),
                    stencil: Default::default(),
                    bias,
                }),
                multisample: wgpu::MultisampleState { count: SAMPLES, ..Default::default() },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(fs),
                    compilation_options: Default::default(),
                    targets: &[Some(format.into())],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        // Flächen leicht nach hinten versetzt, damit die Kanten darauf sauber sichtbar bleiben.
        let mesh_pipeline = pipeline(
            "flächen",
            "vs_mesh",
            "fs_mesh",
            wgpu::PrimitiveTopology::TriangleList,
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<MeshVertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &mesh_attributes,
            },
            wgpu::DepthBiasState { constant: 2, slope_scale: 1.5, clamp: 0.0 },
        );
        let edge_pipeline = pipeline(
            "kanten",
            "vs_edge",
            "fs_edge",
            wgpu::PrimitiveTopology::LineList,
            wgpu::VertexBufferLayout {
                array_stride: 12,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &edge_attributes,
            },
            Default::default(),
        );

        let (msaa, depth) = attachments(&device, format, size);
        let camera = OrbitCamera { view_size: (size.0 as f32 / scale, size.1 as f32 / scale), ..Default::default() };
        Self {
            device,
            queue,
            target,
            format,
            size,
            scale,
            msaa,
            depth,
            globals,
            bind_group,
            mesh_pipeline,
            edge_pipeline,
            buffers: None,
            bounds: None,
            triangles: 0,
            camera,
        }
    }

    pub fn camera(&self) -> &OrbitCamera {
        &self.camera
    }

    pub fn camera_mut(&mut self) -> &mut OrbitCamera {
        &mut self.camera
    }

    pub fn triangle_count(&self) -> u32 {
        self.triangles
    }

    /// Neue Größe in Pixeln und Skalierung (Pixel je Punkt).
    pub fn resize(&mut self, width: u32, height: u32, scale: f32) {
        let (width, height) = (width.max(1), height.max(1));
        self.scale = scale;
        self.camera.view_size = (width as f32 / scale, height as f32 / scale);
        if (width, height) == self.size {
            return;
        }
        self.size = (width, height);
        if let Target::Surface { surface, config } = &mut self.target {
            config.width = width;
            config.height = height;
            surface.configure(&self.device, config);
        }
        (self.msaa, self.depth) = attachments(&self.device, self.format, self.size);
    }

    /// Lädt die Geometrie aller Elemente auf die Grafikkarte.
    pub fn set_geometry(&mut self, geometry: &ModelGeometry) {
        let mut vertices = Vec::new();
        let mut edges: Vec<[f32; 3]> = Vec::new();
        for element in &geometry.elements {
            vertices.extend(
                element
                    .mesh
                    .flat_vertices()
                    .into_iter()
                    .map(|v| MeshVertex { position: v.position, normal: v.normal }),
            );
            for [a, b] in element.mesh.feature_edges(20.0) {
                edges.push(a);
                edges.push(b);
            }
        }
        self.triangles = (vertices.len() / 3) as u32;
        self.bounds = geometry.bounds().map(|(lo, hi)| {
            (Vec3::from_array(lo.map(|v| v as f32)), Vec3::from_array(hi.map(|v| v as f32)))
        });
        let buffer = |label: &str, contents: &[u8]| {
            self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents,
                usage: wgpu::BufferUsages::VERTEX,
            })
        };
        // Leere Puffer sind nicht erlaubt, deshalb mindestens ein Eckpunkt.
        let mesh = buffer("flächen", if vertices.is_empty() { &[0; 24] } else { bytemuck::cast_slice(&vertices) });
        let edge_buffer = buffer("kanten", if edges.is_empty() { &[0; 12] } else { bytemuck::cast_slice(&edges) });
        self.buffers = Some(Buffers {
            mesh,
            mesh_count: vertices.len() as u32,
            edges: edge_buffer,
            edge_count: edges.len() as u32,
        });
    }

    /// Stellt die Kamera auf das ganze Modell ein.
    pub fn zoom_to_fit(&mut self) {
        if let Some((lo, hi)) = self.bounds {
            self.camera.fit((lo + hi) / 2.0, (hi - lo).length() / 2.0);
        }
    }

    /// Zeichnet ein Bild in die Fläche des Views. Gibt `false` zurück, wenn
    /// kein Bild entstand, etwa weil das Fenster verdeckt ist oder auf einem
    /// anderen Space liegt. Dann später erneut versuchen.
    pub fn render(&mut self) -> Result<bool, RenderError> {
        let frame = match &self.target {
            Target::Surface { surface, config } => match surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
                wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => return Ok(false),
                wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                    surface.configure(&self.device, config);
                    return Ok(false);
                }
                other => return Err(RenderError::Surface(format!("{other:?}"))),
            },
            Target::Offscreen { .. } => {
                self.draw_offscreen();
                return Ok(true);
            }
        };
        let view = frame.texture.create_view(&Default::default());
        self.draw(&view);
        self.queue.present(frame);
        Ok(true)
    }

    /// Zeichnet ein Bild ohne Fenster und gibt es als RGBA-Pixel zurück (Zeilen von oben).
    pub fn render_image(&mut self) -> Result<Vec<u8>, RenderError> {
        let Target::Offscreen { texture } = &self.target else {
            return Err(RenderError::Surface("Nur ohne Fenster möglich".into()));
        };
        let texture = texture.clone();
        self.draw_offscreen();
        let (width, height) = self.size;
        let row = width * 4;
        let padded = row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: (padded * height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(height),
                },
            },
            extent(width, height),
        );
        self.queue.submit([encoder.finish()]);
        let slice = readback.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| RenderError::Device(e.to_string()))?;
        let data = slice.get_mapped_range().map_err(|e| RenderError::Device(format!("{e:?}")))?;
        let mut pixels = Vec::with_capacity((row * height) as usize);
        for y in 0..height {
            let start = (y * padded) as usize;
            pixels.extend_from_slice(&data[start..start + row as usize]);
        }
        Ok(pixels)
    }

    fn draw_offscreen(&mut self) {
        if let Target::Offscreen { texture } = &self.target {
            let view = texture.create_view(&Default::default());
            self.draw(&view);
        }
    }

    fn draw(&self, target: &wgpu::TextureView) {
        let eye = self.camera.eye();
        // Licht von schräg links oben hinter der Kamera, damit die Flächen sich unterscheiden.
        let forward = (self.camera.target - eye).normalize_or_zero();
        let right = forward.cross(Vec3::Z).normalize_or_zero();
        let light = (-forward - 0.6 * right + 0.8 * Vec3::Z).normalize_or_zero();
        let globals = Globals {
            view_proj: self.camera.view_proj().to_cols_array_2d(),
            light: [light.x, light.y, light.z, 0.0],
            base: srgb_to_linear(BASE),
            edge: srgb_to_linear(EDGE),
        };
        self.queue.write_buffer(&self.globals, 0, bytemuck::bytes_of(&globals));

        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let [r, g, b] = BACKGROUND.map(|c| srgb_component_to_linear(c as f32) as f64);
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("3d"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.msaa,
                    depth_slice: None,
                    resolve_target: Some(target),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r, g, b, a: 1.0 }),
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
            if let Some(buffers) = &self.buffers {
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.set_pipeline(&self.mesh_pipeline);
                pass.set_vertex_buffer(0, buffers.mesh.slice(..));
                pass.draw(0..buffers.mesh_count, 0..1);
                pass.set_pipeline(&self.edge_pipeline);
                pass.set_vertex_buffer(0, buffers.edges.slice(..));
                pass.draw(0..buffers.edge_count, 0..1);
            }
        }
        self.queue.submit([encoder.finish()]);
    }
}

fn open_device(
    instance: &wgpu::Instance,
    surface: Option<&wgpu::Surface<'_>>,
) -> Result<(wgpu::Adapter, wgpu::Device, wgpu::Queue), RenderError> {
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: surface,
        apply_limit_buckets: false,
    }))
    .map_err(|e| RenderError::NoAdapter(e.to_string()))?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("lot"),
        required_limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
        ..Default::default()
    }))
    .map_err(|e| RenderError::Device(e.to_string()))?;
    Ok((adapter, device, queue))
}

fn attachments(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    (width, height): (u32, u32),
) -> (wgpu::TextureView, wgpu::TextureView) {
    let texture = |label: &str, format: wgpu::TextureFormat| {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: extent(width, height),
                mip_level_count: 1,
                sample_count: SAMPLES,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&Default::default())
    };
    (texture("msaa", format), texture("tiefe", DEPTH_FORMAT))
}

fn extent(width: u32, height: u32) -> wgpu::Extent3d {
    wgpu::Extent3d { width: width.max(1), height: height.max(1), depth_or_array_layers: 1 }
}

fn srgb_component_to_linear(c: f32) -> f32 {
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

fn srgb_to_linear(c: [f32; 3]) -> [f32; 4] {
    let [r, g, b] = c.map(srgb_component_to_linear);
    [r, g, b, 1.0]
}
