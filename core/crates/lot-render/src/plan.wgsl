// Shader der Planansicht: Dreiecke in Blattmillimetern mit Farbe je Eckpunkt.

struct PlanGlobals {
    // Blattmillimeter nach Bildschirm (Clip-Raum).
    transform: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> g: PlanGlobals;

struct PlanOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_plan(@location(0) position: vec2<f32>, @location(1) color: vec4<f32>) -> PlanOut {
    var out: PlanOut;
    out.position = g.transform * vec4<f32>(position, 0.0, 1.0);
    out.color = color;
    return out;
}

@fragment
fn fs_plan(in: PlanOut) -> @location(0) vec4<f32> {
    return in.color;
}
