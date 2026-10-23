// Unterlagen unter der Planansicht: ein Bild je Viereck, blass über Weiß.

struct PlanGlobals {
    // Blattmillimeter nach Bildschirm (Clip-Raum).
    transform: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> g: PlanGlobals;
@group(0) @binding(1) var image: texture_2d<f32>;
@group(0) @binding(2) var image_sampler: sampler;

struct UnderlayOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) alpha: f32,
};

@vertex
fn vs_underlay(@location(0) position: vec2<f32>, @location(1) uv: vec2<f32>, @location(2) alpha: f32) -> UnderlayOut {
    var out: UnderlayOut;
    out.position = g.transform * vec4<f32>(position, 0.0, 1.0);
    out.uv = uv;
    out.alpha = alpha;
    return out;
}

@fragment
fn fs_underlay(in: UnderlayOut) -> @location(0) vec4<f32> {
    let c = textureSample(image, image_sampler, in.uv);
    return vec4<f32>(c.rgb, c.a * in.alpha);
}
