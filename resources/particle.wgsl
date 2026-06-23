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

    let p = (in.uv - vec2<f32>(0.5, 0.5)) * 2.0;

    let r2 = dot(p, p);

    if (r2 > 1.0) {
        discard;
    }

    let z = sqrt(1.0-r2);

    let normal =
        normalize(vec3<f32>(p.x, p.y, z));

    let light_dir =
        normalize(vec3<f32>(1.0, 1.0, 1.0));

    let brightness =
        max(dot(normal, light_dir), 0.1);

    return vec4<f32>(
        in.color * brightness,
        1.0
    );

}
