//! Renderer auf Basis von wgpu: 3D-Ansicht und Planansicht.
//!
//! Läuft auf dem Mac über Metal und zeichnet in den CAMetalLayer eines NSView
//! der App ([`Renderer::for_appkit_view`]). Ohne Fenster zeichnet er in eine
//! Textur und liefert ein Bild ([`Renderer::headless`]), etwa für Prüfpakete
//! und Tests. Plattformabhängig ist nur, welches Handle wgpu bekommt; der
//! Code hier baut auch unter Linux.

mod camera;
mod plan2d;
mod underlay;

use std::ffi::c_void;
use std::ptr::NonNull;

use bytemuck::{Pod, Zeroable};
pub use camera::OrbitCamera;
use glam::Vec3;
use lot_geom::ModelGeometry;
use lot_plan::PlanObject;
pub use plan2d::{OverlayItem, PlanCamera};
pub use underlay::{UnderlayImage, UnderlayQuad};
use wgpu::util::DeviceExt;

/// Was der Renderer zeigt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Model,
    Plan,
}

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
/// Hintergrund und Kanten in sRGB.
const BACKGROUND: [f64; 3] = [0.94, 0.94, 0.93];
const EDGE: [f32; 3] = [0.22, 0.22, 0.24];

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Globals {
    view_proj: [[f32; 4]; 4],
    light: [f32; 4],
    edge: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct MeshVertex {
    position: [f32; 3],
    normal: [f32; 3],
    /// Farbe des Materials, linear.
    color: [f32; 3],
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

/// Dreiecke der Planansicht auf der Grafikkarte.
struct PlanBuffers {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    count: u32,
}

/// Alles für die Planansicht.
struct PlanState {
    pipeline: wgpu::RenderPipeline,
    globals: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    objects: Vec<PlanObject>,
    bounds: Option<([f64; 2], [f64; 2])>,
    /// Zerlegte Planobjekte und der Zoom, für den sie zerlegt wurden.
    mesh: Option<(PlanBuffers, f64)>,
    overlay: Vec<OverlayItem>,
    camera: PlanCamera,
    underlay_pipeline: underlay::UnderlayPipeline,
    underlays: Vec<underlay::UnderlayGpu>,
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
    mode: ViewMode,
    plan: PlanState,
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
        let surface =
            unsafe { instance.create_surface_unsafe(target) }.map_err(|e| RenderError::Surface(e.to_string()))?;
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

        let mesh_attributes = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x3];
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

        let plan = plan_state(&device, format);
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
            mode: ViewMode::Model,
            plan,
        }
    }

    pub fn mode(&self) -> ViewMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: ViewMode) {
        self.mode = mode;
    }

    /// Planobjekte für die Planansicht, in Blattmillimetern.
    pub fn set_plan(&mut self, objects: Vec<PlanObject>) {
        self.plan.bounds = plan_bounds(&objects);
        self.plan.objects = objects;
        self.plan.mesh = None;
    }

    /// Unterlagen unter dem Plan (6.05), Ecken in Blattmillimetern.
    pub fn set_underlays(&mut self, quads: &[UnderlayQuad]) {
        let old = std::mem::take(&mut self.plan.underlays);
        self.plan.underlays =
            underlay::update(&self.device, &self.queue, &self.plan.underlay_pipeline, &self.plan.globals, old, quads);
    }

    /// Hinweise über dem Plan (Auswahl, Vorschau, Fangpunkt).
    pub fn set_overlay(&mut self, items: Vec<OverlayItem>) {
        self.plan.overlay = items;
    }

    pub fn plan_camera(&self) -> PlanCamera {
        self.plan.camera
    }

    pub fn set_plan_camera(&mut self, camera: PlanCamera) {
        self.plan.camera = camera;
    }

    /// Pixel je Punkt (Retina: 2).
    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// Verschiebt die Planansicht um eine Fingerbewegung in Punkten.
    pub fn plan_pan(&mut self, dx: f32, dy: f32) {
        let mm_per_pt = self.scale as f64 / self.plan.camera.px_per_mm;
        self.plan.camera.center[0] -= dx as f64 * mm_per_pt;
        self.plan.camera.center[1] += dy as f64 * mm_per_pt;
    }

    /// Zoomt die Planansicht zum Zeiger (Lage in Punkten von der Mitte, y nach oben).
    pub fn plan_zoom(&mut self, factor: f32, x: f32, y: f32) {
        if !(factor.is_finite() && factor > 0.0) {
            return;
        }
        let before = self.plan_point(x, y);
        let zoom = (self.plan.camera.px_per_mm * factor as f64).clamp(0.05, 400.0);
        self.plan.camera.px_per_mm = zoom;
        let after = self.plan_point(x, y);
        self.plan.camera.center[0] += before[0] - after[0];
        self.plan.camera.center[1] += before[1] - after[1];
    }

    /// Blattmillimeter unter einem Zeiger (Lage in Punkten von der Mitte, y nach oben).
    pub fn plan_point(&self, x: f32, y: f32) -> [f64; 2] {
        let mm_per_pt = self.scale as f64 / self.plan.camera.px_per_mm;
        [self.plan.camera.center[0] + x as f64 * mm_per_pt, self.plan.camera.center[1] + y as f64 * mm_per_pt]
    }

    /// Blattmillimeter je Punkt beim aktuellen Zoom.
    pub fn plan_mm_per_point(&self) -> f64 {
        self.scale as f64 / self.plan.camera.px_per_mm
    }

    /// Stellt die Planansicht auf den ganzen Plan ein.
    pub fn plan_zoom_to_fit(&mut self) {
        let Some((lo, hi)) = self.plan.bounds else { return };
        let (w, h) = ((hi[0] - lo[0]).max(1.0), (hi[1] - lo[1]).max(1.0));
        let (vw, vh) = (self.size.0 as f64, self.size.1 as f64);
        self.plan.camera.px_per_mm = (vw / w).min(vh / h) * 0.85;
        self.plan.camera.center = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
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

    /// Lädt die Geometrie aller Elemente auf die Grafikkarte. `color` liefert
    /// zu jedem Materialschlüssel die Farbe in sRGB.
    pub fn set_geometry(&mut self, geometry: &ModelGeometry, color: &dyn Fn(&str) -> [u8; 3]) {
        let mut vertices = Vec::new();
        let mut edges: Vec<[f32; 3]> = Vec::new();
        for element in &geometry.elements {
            let [r, g, b, _] = srgb_to_linear(color(&element.material).map(|c| c as f32 / 255.0));
            vertices.extend(element.mesh.flat_vertices().into_iter().map(|v| MeshVertex {
                position: v.position,
                normal: v.normal,
                color: [r, g, b],
            }));
            for [a, b] in element.mesh.feature_edges(20.0) {
                edges.push(a);
                edges.push(b);
            }
        }
        self.triangles = (vertices.len() / 3) as u32;
        self.bounds = geometry
            .bounds()
            .map(|(lo, hi)| (Vec3::from_array(lo.map(|v| v as f32)), Vec3::from_array(hi.map(|v| v as f32))));
        let buffer = |label: &str, contents: &[u8]| {
            self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents,
                usage: wgpu::BufferUsages::VERTEX,
            })
        };
        // Leere Puffer sind nicht erlaubt, deshalb mindestens ein Eckpunkt.
        let mesh = buffer("flächen", if vertices.is_empty() { &[0; 36] } else { bytemuck::cast_slice(&vertices) });
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
        self.draw_mode(&view);
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
        self.device.poll(wgpu::PollType::wait_indefinitely()).map_err(|e| RenderError::Device(e.to_string()))?;
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
            self.draw_mode(&view);
        }
    }

    fn draw_mode(&mut self, target: &wgpu::TextureView) {
        match self.mode {
            ViewMode::Model => self.draw(target),
            ViewMode::Plan => self.draw_plan(target),
        }
    }

    fn draw_plan(&mut self, target: &wgpu::TextureView) {
        let zoom = self.plan.camera.px_per_mm;
        // Neu zerlegen, wenn sich der Zoom spürbar geändert hat: Linien bleiben mindestens 1 Pixel breit.
        let stale = self.plan.mesh.as_ref().is_none_or(|(_, used)| (zoom / used - 1.0).abs() > 0.05);
        if stale {
            let mesh = plan2d::tessellate(&self.plan.objects, (1.0 / zoom) as f32);
            self.plan.mesh = Some((upload(&self.device, &mesh), zoom));
        }
        let overlay = upload(&self.device, &plan2d::tessellate_overlay(&self.plan.overlay, zoom));

        let (w, h) = (self.size.0 as f64, self.size.1 as f64);
        let [cx, cy] = self.plan.camera.center;
        let sx = 2.0 * zoom / w;
        let sy = 2.0 * zoom / h;
        let transform = [
            [sx as f32, 0.0, 0.0, 0.0],
            [0.0, sy as f32, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [(-cx * sx) as f32, (-cy * sy) as f32, 0.0, 1.0],
        ];
        self.queue.write_buffer(&self.plan.globals, 0, bytemuck::bytes_of(&transform));

        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("plan"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.msaa,
                    depth_slice: None,
                    resolve_target: Some(target),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Discard,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            underlay::draw(&mut pass, &self.plan.underlay_pipeline, &self.plan.underlays);
            pass.set_pipeline(&self.plan.pipeline);
            pass.set_bind_group(0, &self.plan.bind_group, &[]);
            for buffers in self.plan.mesh.iter().map(|(b, _)| b).chain(std::iter::once(&overlay)) {
                if buffers.count > 0 {
                    pass.set_vertex_buffer(0, buffers.vertices.slice(..));
                    pass.set_index_buffer(buffers.indices.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..buffers.count, 0, 0..1);
                }
            }
        }
        self.queue.submit([encoder.finish()]);
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
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Discard }),
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

fn plan_state(device: &wgpu::Device, format: wgpu::TextureFormat) -> PlanState {
    let shader = device.create_shader_module(wgpu::include_wgsl!("plan.wgsl"));
    let globals = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("plan"),
        size: 64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("plan"),
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
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("plan"),
        layout: &layout,
        entries: &[wgpu::BindGroupEntry { binding: 0, resource: globals.as_entire_binding() }],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("plan"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let attributes = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4];
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("plan"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_plan"),
            compilation_options: Default::default(),
            buffers: &[Some(wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<plan2d::PlanVertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &attributes,
            })],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState { count: SAMPLES, ..Default::default() },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_plan"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    });
    PlanState {
        pipeline,
        globals,
        bind_group,
        objects: Vec::new(),
        bounds: None,
        mesh: None,
        overlay: Vec::new(),
        camera: PlanCamera::default(),
        underlay_pipeline: underlay::pipeline(device, format, SAMPLES),
        underlays: Vec::new(),
    }
}

fn upload(device: &wgpu::Device, mesh: &plan2d::PlanMesh) -> PlanBuffers {
    let vertices: &[u8] = if mesh.vertices.is_empty() {
        &[0; std::mem::size_of::<plan2d::PlanVertex>()]
    } else {
        bytemuck::cast_slice(&mesh.vertices)
    };
    let indices: &[u8] = if mesh.indices.is_empty() { &[0; 4] } else { bytemuck::cast_slice(&mesh.indices) };
    PlanBuffers {
        vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("plan"),
            contents: vertices,
            usage: wgpu::BufferUsages::VERTEX,
        }),
        indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("plan"),
            contents: indices,
            usage: wgpu::BufferUsages::INDEX,
        }),
        count: mesh.indices.len() as u32,
    }
}

/// Hüllrechteck aller Planobjekte in Blattmillimetern.
fn plan_bounds(objects: &[PlanObject]) -> Option<([f64; 2], [f64; 2])> {
    let mut points = objects.iter().flat_map(|o| -> Vec<lot_plan::PaperPoint> {
        match o {
            PlanObject::Line(l) => l.points.clone(),
            PlanObject::Area(a) => a.rings.iter().flatten().copied().collect(),
            PlanObject::Text(t) => vec![t.position],
        }
    });
    let first = points.next()?;
    Some(points.fold(([first.x, first.y], [first.x, first.y]), |(lo, hi), p| {
        ([lo[0].min(p.x), lo[1].min(p.y)], [hi[0].max(p.x), hi[1].max(p.y)])
    }))
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Zeichnet die Beispielwand ohne Fenster. Ohne Grafikkarte (etwa in
    /// Cloud-Sessions) wird der Test übersprungen.
    #[test]
    fn headless_renders_wall() {
        let mut renderer = match Renderer::headless(320, 200) {
            Ok(r) => r,
            Err(RenderError::NoAdapter(e)) => {
                eprintln!("übersprungen, keine Grafikkarte: {e}");
                return;
            }
            Err(e) => panic!("{e}"),
        };
        let geometry = lot_geom::build(&lot_model::Project::sample()).unwrap();
        renderer.set_geometry(&geometry, &|_| [235, 233, 226]);
        renderer.zoom_to_fit();
        let pixels = renderer.render_image().unwrap();
        assert_eq!(pixels.len(), 320 * 200 * 4);

        let background = BACKGROUND.map(|c| (c * 255.0).round() as i32);
        let pixel = |x: usize, y: usize| {
            let i = (y * 320 + x) * 4;
            [pixels[i] as i32, pixels[i + 1] as i32, pixels[i + 2] as i32]
        };
        let differs = |p: [i32; 3]| p.iter().zip(background).any(|(a, b)| (a - b).abs() > 3);
        // Die Ecke bleibt Hintergrund, im Bild ist die Wand zu sehen, mit dunklen Kanten.
        assert!(!differs(pixel(0, 0)), "Ecke ist nicht Hintergrund: {:?}", pixel(0, 0));
        let covered =
            (0..200).flat_map(|y| (0..320).map(move |x| (x, y))).filter(|&(x, y)| differs(pixel(x, y))).count();
        assert!(covered > 320 * 200 / 10, "Wand bedeckt nur {covered} Pixel");
        let dark = (0..200)
            .flat_map(|y| (0..320).map(move |x| (x, y)))
            .filter(|&(x, y)| pixel(x, y).iter().all(|c| *c < 120))
            .count();
        assert!(dark > 50, "kaum Kanten: {dark} dunkle Pixel");
    }

    /// Eine rote Unterlage in der Planansicht färbt die Mitte blassrot; daneben bleibt Weiß.
    #[test]
    fn underlay_shows_faded_under_the_plan() {
        let mut renderer = match Renderer::headless(200, 100) {
            Ok(r) => r,
            Err(RenderError::NoAdapter(e)) => {
                eprintln!("übersprungen, keine Grafikkarte: {e}");
                return;
            }
            Err(e) => panic!("{e}"),
        };
        renderer.set_mode(ViewMode::Plan);
        let image = std::sync::Arc::new(UnderlayImage { width: 2, height: 2, rgba: [255, 0, 0, 255].repeat(4) });
        renderer.set_underlays(&[UnderlayQuad {
            corners: [[-10.0, -10.0], [10.0, -10.0], [10.0, 10.0], [-10.0, 10.0]],
            image,
            opacity: 0.5,
        }]);
        renderer.set_plan_camera(PlanCamera { center: [0.0, 0.0], px_per_mm: 2.0 });
        let pixels = renderer.render_image().unwrap();
        let pixel = |x: usize, y: usize| {
            let i = (y * 200 + x) * 4;
            [pixels[i], pixels[i + 1], pixels[i + 2]]
        };
        let mid = pixel(100, 50);
        assert!(mid[0] > 240 && mid[1] > 120 && mid[1] < 230 && mid[2] < 230, "{mid:?}");
        assert_eq!(pixel(5, 5), [255, 255, 255]);
    }
}
