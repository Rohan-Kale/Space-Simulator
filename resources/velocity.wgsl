struct Camera {
    view_proj: mat4x4<f32>,
};

struct GpuBody {
    position: vec4<f32>,
    velocity: vec4<f32>,
    acceleration: vec4<f32>,
    mass: f32,
    radius: f32,
    _pad: vec2<f32>,
};

@group(0) @binding(0)
var<storage, read> bodies: array<GpuBody>;

@group(1) @binding(0)
var<uniform> camera: Camera;

struct VSOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

fn safe_vec3(v: vec3<f32>) -> vec3<f32> {
    return select(v, vec3<f32>(0.0), (any(v != v)));
}

fn clamp_vec3(v: vec3<f32>, minv: f32, maxv: f32) -> vec3<f32> {
    return clamp(v, vec3<f32>(minv), vec3<f32>(maxv));
}

@vertex
fn vs_main(
    @builtin(vertex_index) v: u32,
    @builtin(instance_index) i: u32
) -> VSOut {

    let body = bodies[i];

    var pos = safe_vec3(body.position.xyz);
    var vel = safe_vec3(body.velocity.xyz);

    // Prevent insane debug lines
    let speed = length(vel);

    // HARD clamp velocity for visualization only
    let clamped_vel = clamp_vec3(vel, -10.0, 10.0);

    // Scale line length (keep small to avoid “laser beams”)
    let scale = 0.2;

    let start = pos;
    let end = pos + clamped_vel * scale;

    var world_pos = select(start, end, v == 1u);

    var out: VSOut;
    out.position = camera.view_proj * vec4<f32>(world_pos, 1.0);

    // Color by speed (stable range)
    let t = clamp(speed * 0.05, 0.0, 1.0);

    out.color = vec3<f32>(
        t,
        0.3,
        1.0 - t
    );

    return out;
}

@fragment
fn fs_main(input: VSOut) -> @location(0) vec4<f32> {
    return vec4<f32>(input.color, 1.0);
}
