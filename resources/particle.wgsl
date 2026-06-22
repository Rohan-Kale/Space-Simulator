struct Camera {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @location(0) pos: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) uv: vec2<f32>,

    @location(3) offset: vec3<f32>,
    @location(4) radius: f32,
) -> VertexOutput {

    var out: VertexOutput;

    // Scale the quad
    let local_pos = pos * radius;

    // Move into world space
    let world_pos = local_pos + offset;

    // Apply camera
    out.position =
        camera.view_proj *
        vec4<f32>(world_pos, 1.0);

    out.color = color;
    out.uv = uv;

    return out;
}

@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {

    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(in.uv, center);

    if (dist > 0.5) {
        discard;
    }

    return vec4<f32>(in.color, 1.0);
}
