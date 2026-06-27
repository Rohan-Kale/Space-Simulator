struct Body {
    position: vec4<f32>,
    velocity: vec4<f32>,
    acceleration: vec4<f32>,
    mass: f32,
    radius: f32,
    padding: vec2<f32>,
}

@group(0) @binding(0)
var<storage, read_write> bodies: array<Body>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {

    let i = id.x;

    if(i >= arrayLength(&bodies)){
        return;
    }

    var acceleration = vec3<f32>(0.0);

    for(var j:u32 = 0u; j < arrayLength(&bodies); j++){
        if(i == j) { continue; }
        
        let dir = bodies[j].position.xyz - bodies[i].position.xyz;
        let dist = length(dir) + 0.01;
        let force = bodies[j].mass / (dist * dist);
        acceleration += normalize(dir) * force;
    }
    let new_velocity = bodies[i].velocity.xyz + acceleration * 0.001;
    let new_position = bodies[i].position.xyz + new_velocity * 0.001;

    bodies[i].velocity = vec4<f32>(new_velocity, 0.0);
    bodies[i].position = vec4<f32>(new_position, 0.0);
}
