struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    @location(0) pos: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) offset: vec2<f32>,
    @location(3) radius: f32,
) -> VertexOutput {

    var out: VertexOutput;

    let scaled_pos = pos.xy * radius;

    out.position = vec4<f32>(
        scaled_pos + offset,
        0.0,
        1.0
    );

    out.color = color;

    return out;
}

@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {

    return vec4<f32>(
        in.color,
        1.0
    );
}
