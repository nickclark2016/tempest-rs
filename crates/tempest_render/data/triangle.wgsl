struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vert_pos: vec3<f32>,
    @location(1) vert_color : vec3<f32>
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    var vertex_positions: array<vec4<f32>, 3> = array<vec4<f32>, 3>(
        vec4<f32>(-1.0, -1.0, 0.0, 1.0),
        vec4<f32>(1.0, -1.0, 0.0, 1.0),
        vec4<f32>(0.0, 1.0, 0.0, 1.0),
    );

    var vertex_colors: array<vec3<f32>, 3> = array<vec3<f32>, 3>(
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(0.0, 0.0, 1.0),
    );

    out.clip_position = vec4<f32>(vertex_positions[in_vertex_index].xyz / 2.0, 1.0);
    out.vert_pos = vertex_positions[in_vertex_index].xyz;
    out.vert_color = vertex_colors[in_vertex_index];

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.vert_color, 1.0);
}