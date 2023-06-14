struct Object {
    transform: mat4x4<f32>,
    first_index: u32,
    index_count: u32,
    material_index: u32,
    vertex_attribute_start_offsets: array<u32, {{vertex_attrib_count}}>,
}

struct ObjectRange {
    object_id: u32,
}

struct Batch {
    ranges: array<ObjectRange, 256>, // number of batches
    object_count: u32,
}