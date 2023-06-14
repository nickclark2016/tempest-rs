{{include "tempest/structures.wgsl"}}

struct Indices {
    vertex_id: u32,
    object_id: u32,
}

fn extract_attr_v2_f32(byte_offset: u32, vertex_id: u32) -> vec2<f32> {
    let start = byte_offset / 4u + vertex_id * 2u; // get offset to the first vertex in the float stream
    return vec2<f32>(
        bitcast<f32>(vertex_buffer[start]), // x
        bitcast<f32>(vertex_buffer[start + 1u]), // y
    );
}

fn extract_attr_v3_f32(byte_offset: u32, vertex_id: u32) -> vec3<f32> {
    let start = byte_offset / 4u + vertex_id * 3u; // get offset to the first vertex in the float stream
    return vec3<f32>(
        bitcast<f32>(vertex_buffer[start]), // x
        bitcast<f32>(vertex_buffer[start + 1u]), // y
        bitcast<f32>(vertex_buffer[start + 2u]), // z
    );
}

fn extract_attr_v4_f32(byte_offset: u32, vertex_id: u32) -> vec4<f32> {
    let start = byte_offset / 4u + vertex_id * 3u; // get offset to the first vertex in the float stream
    return vec4<f32>(
        bitcast<f32>(vertex_buffer[start]), // x
        bitcast<f32>(vertex_buffer[start + 1u]), // y
        bitcast<f32>(vertex_buffer[start + 2u]), // z
        bitcast<f32>(vertex_buffer[start + 3u]), // w
    );
}