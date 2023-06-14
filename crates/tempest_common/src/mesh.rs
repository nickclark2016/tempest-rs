//! Triangle meshes and associated primitives.
use tempest_math::f32::{vec2::Vec2, vec3::Vec3, vec4::Vec4};

/// Single vertex in a mesh.
#[derive(Clone, Copy)]
pub struct Vertex {
    /// Position coordiate of the vertex
    pub position: Vec3,
    /// Optional texture coordinate
    pub uvcoord: Option<Vec2>,
    /// Optional normal vector
    pub normal: Option<Vec3>,
    /// Optional tangent vector
    pub tangent: Option<Vec3>,
    /// Optional color
    pub color: Option<Vec4>,
    // TODO: Vertex weights for animations
}

/// Triangle with vertices ordered in counter-clockwise order.
#[derive(Clone, Copy)]
pub struct Triangle {
    /// First vertex
    pub v1: Vertex,
    /// Second vertex
    pub v2: Vertex,
    /// Third vertex
    pub v3: Vertex,
}

impl Triangle {
    /// Constructs a new triangle from a set of vertices
    pub fn new(v1: Vertex, v2: Vertex, v3: Vertex) -> Self {
        Self { v1, v2, v3 }
    }

    /// Computes the normal of the triangle's face
    pub fn face_normal(&self) -> Vec3 {
        // (v2 - v1) x (v3 - v1) for CCW
        let d1 = self.v2.position - self.v1.position;
        let d2 = self.v3.position - self.v1.position;
        d1.cross(d2)
    }
}

/// Mesh representation based on triangle list topology.
#[derive(Clone)]
pub struct Mesh {
    /// List of vertices
    pub vertices: Vec<Vertex>,
    /// List of indices
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        assert!(
            indices.len() % 3 == 0,
            "Index count must be a multiple of 3."
        );
        Self { vertices, indices }
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    pub fn get_triangle(&self, index: usize) -> Option<Triangle> {
        if index >= self.triangle_count() {
            None
        } else {
            let v1 = self.vertices[3 * index + 0];
            let v2 = self.vertices[3 * index + 1];
            let v3 = self.vertices[3 * index + 2];
            Some(Triangle::new(v1, v2, v3))
        }
    }
}
