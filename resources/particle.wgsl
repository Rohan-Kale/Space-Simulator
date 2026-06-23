struct Camera {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;


struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) normal: vec3<f32>,
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


    let world_pos = pos * radius + offset;


    out.position =
        camera.view_proj *
        vec4<f32>(world_pos,1.0);


    out.color = color;

    // sphere centered at origin
    out.normal = normalize(pos);


    return out;
}


@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {


    let light_dir =
        normalize(vec3<f32>(1.0,1.0,1.0));


    let brightness =
        max(dot(in.normal, light_dir),0.1);


    return vec4<f32>(
        in.color * brightness,
        1.0
    );
}
