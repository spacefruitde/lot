// Shader der 3D-Ansicht: flach schattierte Flächen und dunkle Kanten.

struct Globals {
    view_proj: mat4x4<f32>,
    // Richtung zum Hauptlicht, im Weltsystem (xyz), w ungenutzt.
    light: vec4<f32>,
    // Kantenfarbe, linear.
    edge: vec4<f32>,
};

@group(0) @binding(0) var<uniform> g: Globals;

struct MeshOut {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) color: vec3<f32>,
};

@vertex
fn vs_mesh(@location(0) position: vec3<f32>, @location(1) normal: vec3<f32>, @location(2) color: vec3<f32>) -> MeshOut {
    var out: MeshOut;
    out.position = g.view_proj * vec4<f32>(position, 1.0);
    out.normal = normal;
    out.color = color;
    return out;
}

@fragment
fn fs_mesh(in: MeshOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let key = max(dot(n, normalize(g.light.xyz)), 0.0);
    // Himmelslicht von oben, damit auch abgewandte Flächen lesbar bleiben.
    let sky = 0.5 + 0.5 * n.z;
    let shade = 0.35 + 0.45 * key + 0.2 * sky;
    return vec4<f32>(in.color * shade, 1.0);
}

@vertex
fn vs_edge(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    return g.view_proj * vec4<f32>(position, 1.0);
}

@fragment
fn fs_edge() -> @location(0) vec4<f32> {
    return g.edge;
}
