struct Body {
    position: vec4<f32>,
    velocity: vec4<f32>,
    acceleration: vec4<f32>,
    mass: f32,
    radius: f32,
    padding: vec2<f32>,
}


struct TrailVertex {
    position: vec4<f32>,
    alpha: f32,
}


@group(0) @binding(0)
var<storage, read> bodies: array<Body>;


@group(0) @binding(1)
var<storage, read_write> trails: array<TrailVertex>;


@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {

    let i = id.x;

    if(i >= arrayLength(&bodies)) {
        return;
    }

    trails[i].position =
        bodies[i].position;

    trails[i].alpha = 1.0;
}
