//! Unterlagen (6.05) in der Planansicht: Bilder als Textur unter dem Plan.
//!
//! Die Bildpunkte kommen von außen: in der App von PDFKit, in Testprojekten aus
//! PNG-Dateien. Gleiche Bilder werden nur einmal hochgeladen; ändert sich nur
//! die Lage (Kalibrieren, Verschieben), werden nur die vier Ecken neu gesetzt.

use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

/// Bild mit 8 Bit je Kanal, RGBA (sRGB), Zeilen von oben nach unten.
#[derive(Debug, Clone, PartialEq)]
pub struct UnderlayImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Ein Bild an seinem Platz: Ecken in Blattmillimetern (unten links, unten
/// rechts, oben rechts, oben links) und Deckkraft.
#[derive(Debug, Clone)]
pub struct UnderlayQuad {
    pub corners: [[f64; 2]; 4],
    pub image: Arc<UnderlayImage>,
    pub opacity: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
    alpha: f32,
}

pub(crate) struct UnderlayPipeline {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

/// Ein hochgeladenes Bild mit seinen Ecken.
pub(crate) struct UnderlayGpu {
    image: Arc<UnderlayImage>,
    bind_group: wgpu::BindGroup,
    vertices: wgpu::Buffer,
}

pub(crate) fn pipeline(device: &wgpu::Device, format: wgpu::TextureFormat, samples: u32) -> UnderlayPipeline {
    let shader = device.create_shader_module(wgpu::include_wgsl!("underlay.wgsl"));
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("unterlage"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("unterlage"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let attributes = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32];
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("unterlage"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_underlay"),
            compilation_options: Default::default(),
            buffers: &[Some(wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &attributes,
            })],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState { count: samples, ..Default::default() },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_underlay"),
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
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("unterlage"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
    });
    UnderlayPipeline { pipeline, layout, sampler }
}

/// Bringt die Unterlagen auf die Grafikkarte; Bilder, die schon oben sind, bleiben.
pub(crate) fn update(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &UnderlayPipeline,
    globals: &wgpu::Buffer,
    old: Vec<UnderlayGpu>,
    quads: &[UnderlayQuad],
) -> Vec<UnderlayGpu> {
    let mut old = old;
    quads
        .iter()
        .map(|q| {
            let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("unterlage"),
                contents: bytemuck::cast_slice(&vertices(q)),
                usage: wgpu::BufferUsages::VERTEX,
            });
            match old.iter().position(|g| Arc::ptr_eq(&g.image, &q.image)) {
                Some(i) => {
                    let g = old.swap_remove(i);
                    UnderlayGpu { vertices, ..g }
                }
                None => {
                    let view = texture(device, queue, &q.image);
                    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("unterlage"),
                        layout: &p.layout,
                        entries: &[
                            wgpu::BindGroupEntry { binding: 0, resource: globals.as_entire_binding() },
                            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&view) },
                            wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&p.sampler) },
                        ],
                    });
                    UnderlayGpu { image: q.image.clone(), bind_group, vertices }
                }
            }
        })
        .collect()
}

pub(crate) fn draw(pass: &mut wgpu::RenderPass<'_>, p: &UnderlayPipeline, underlays: &[UnderlayGpu]) {
    if underlays.is_empty() {
        return;
    }
    pass.set_pipeline(&p.pipeline);
    for u in underlays {
        pass.set_bind_group(0, &u.bind_group, &[]);
        pass.set_vertex_buffer(0, u.vertices.slice(..));
        pass.draw(0..6, 0..1);
    }
}

/// Zwei Dreiecke; die Bildzeilen laufen von oben nach unten, v = 0 ist oben.
fn vertices(q: &UnderlayQuad) -> [Vertex; 6] {
    let [ll, lr, ur, ul] = q.corners.map(|c| [c[0] as f32, c[1] as f32]);
    let v = |position: [f32; 2], uv: [f32; 2]| Vertex { position, uv, alpha: q.opacity };
    [v(ll, [0.0, 1.0]), v(lr, [1.0, 1.0]), v(ur, [1.0, 0.0]), v(ll, [0.0, 1.0]), v(ur, [1.0, 0.0]), v(ul, [0.0, 0.0])]
}

/// Textur mit allen Mip-Stufen, bei Bedarf verkleinert auf das, was die Grafikkarte kann.
fn texture(device: &wgpu::Device, queue: &wgpu::Queue, image: &UnderlayImage) -> wgpu::TextureView {
    let max = device.limits().max_texture_dimension_2d;
    let mut level = (image.width.max(1), image.height.max(1), image.rgba.clone());
    if level.2.len() != (level.0 * level.1 * 4) as usize {
        level = (1, 1, vec![255, 255, 255, 0]);
    }
    while level.0 > max || level.1 > max {
        level = half(&level);
    }
    let levels = 32 - level.0.max(level.1).leading_zeros();
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("unterlage"),
        size: wgpu::Extent3d { width: level.0, height: level.1, depth_or_array_layers: 1 },
        mip_level_count: levels,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    for mip in 0..levels {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: mip,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &level.2,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(level.0 * 4), rows_per_image: Some(level.1) },
            wgpu::Extent3d { width: level.0, height: level.1, depth_or_array_layers: 1 },
        );
        if mip + 1 < levels {
            level = half(&level);
        }
    }
    texture.create_view(&Default::default())
}

/// Halbe Größe, je 2 × 2 Bildpunkte gemittelt.
fn half((w, h, px): &(u32, u32, Vec<u8>)) -> (u32, u32, Vec<u8>) {
    let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
    let mut out = vec![0u8; (nw * nh * 4) as usize];
    for y in 0..nh {
        for x in 0..nw {
            for c in 0..4 {
                let mut sum = 0u32;
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    let (sx, sy) = ((x * 2 + dx).min(w - 1), (y * 2 + dy).min(h - 1));
                    sum += px[((sy * w + sx) * 4 + c) as usize] as u32;
                }
                out[((y * nw + x) * 4 + c) as usize] = (sum / 4) as u8;
            }
        }
    }
    (nw, nh, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halving_averages_and_keeps_odd_sizes() {
        let px = vec![
            0, 0, 0, 255, 100, 100, 100, 255, 200, 200, 200, 255, //
            0, 0, 0, 255, 100, 100, 100, 255, 200, 200, 200, 255,
        ];
        let (w, h, out) = half(&(3, 2, px));
        assert_eq!((w, h), (1, 1));
        assert_eq!(out, vec![50, 50, 50, 255]);
    }

    #[test]
    fn image_rows_run_from_the_top() {
        let q = UnderlayQuad {
            corners: [[0.0, 0.0], [10.0, 0.0], [10.0, 5.0], [0.0, 5.0]],
            image: Arc::new(UnderlayImage { width: 1, height: 1, rgba: vec![0; 4] }),
            opacity: 0.5,
        };
        let v = vertices(&q);
        // Unten links ist die letzte Bildzeile.
        assert_eq!(v[0].uv, [0.0, 1.0]);
        assert_eq!(v[5].position, [0.0, 5.0]);
        assert_eq!(v[5].uv, [0.0, 0.0]);
    }
}
