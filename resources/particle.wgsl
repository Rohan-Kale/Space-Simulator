struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @location(0) pos: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) uv: vec2<f32>,       // vertex data

    @location(3) offset: vec2<f32>,
    @location(4) radius: f32,
) -> VertexOutput {

    var out: VertexOutput;

    let scaled_pos = pos.xy * radius;

    out.position = vec4<f32>(
        scaled_pos + offset,
        0.0,
        1.0
    );

    out.color = color;
    out.uv = uv;
    return out;
}

@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {

    let center = vec2<f32>(0.5,0.5);

    let dist = distance(in.uv, center);

    if dist > 0.5 {
        discard;
    }

    return vec4<f32>(
        in.color,
        1.0
    );
}
