//! Types required for rendering objects in the world.

use std::{
    marker::PhantomData,
    ops::Deref,
    sync::atomic::{AtomicUsize, Ordering},
};

use bytemuck::Pod;
use once_cell::sync::OnceCell;
use tempest_math::f32::{vec2::Vec2, vec3::Vec3, vec4::Vec4};

/// Trait for renderable materials
pub trait Material: Send + Sync + 'static {
    /// Required vertex attributes
    fn required_attributes() -> Vec<&'static VertexAttributeId>;

    /// Sorting requirements for the material
    fn sorting(&self) -> Sorting;
}

/// Describes how a renderable object is to be sorted
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Sorting {
    /// Order that objects should be sorted in
    pub order: SortOrder,

    /// Reason for sorting to occur
    pub reason: SortRequirement,
}

impl Sorting {
    /// Default sorting operation for opaque objects
    pub const OPAQUE: Self = Self {
        order: SortOrder::FrontToBack,
        reason: SortRequirement::Optimization,
    };

    /// Default sorting operation for objects with transparency requiring sort
    pub const BLEND: Self = Self {
        order: SortOrder::BackToFront,
        reason: SortRequirement::Required,
    };
}

/// Order that renderable objects are to be sorted in relative to the camera
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SortOrder {
    /// Sort the renderable objects in order of ascending distance from camera
    FrontToBack,

    /// Sort the renderable objects in order of descending distance from camera
    BackToFront,
}

/// Requirements for sorting the objects
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SortRequirement {
    /// Optimization related sorting.  Sorts for optimization may result in faster execution but may also be ignored
    Optimization,
    /// Sorting is required for correct rasterization
    Required,
}

/// Represents a unique identifier for a single attribute in the vertex input stream
#[derive(Clone, Copy, Debug)]
pub struct VertexAttributeId {
    inner: usize,
    name: &'static str,
    meta: &'static VertexFormatMeta,
}

impl PartialEq for VertexAttributeId {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Eq for VertexAttributeId {}

impl VertexAttributeId {
    /// Gets the name of the attribute
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Gets the attribute's metadata
    pub fn meta(&self) -> &'static VertexFormatMeta {
        self.meta
    }
}

/// Representation of a vertex attribute data format
pub trait VertexFormat: Pod + Send + Sync + 'static {
    /// Metadata of the vertex attribute data format
    const META: VertexFormatMeta;
}

/// Vertex attribute data format
#[derive(Clone, Copy, Debug)]
pub struct VertexFormatMeta {
    pub size: u32,
    pub data_type: &'static str,
    pub extracter: &'static str,
}

impl VertexFormat for Vec2 {
    const META: VertexFormatMeta = VertexFormatMeta {
        size: 8,
        data_type: "vec2<f32>",
        extracter: "extract_attr_v2_f32",
    };
}

impl VertexFormat for Vec3 {
    const META: VertexFormatMeta = VertexFormatMeta {
        size: 12,
        data_type: "vec3<f32>",
        extracter: "extract_attr_v3_f32",
    };
}

impl VertexFormat for Vec4 {
    const META: VertexFormatMeta = VertexFormatMeta {
        size: 16,
        data_type: "vec4<f32>",
        extracter: "extract_attr_v4_f32",
    };
}

/// Representation of a single vertex attribute in a vertex input stream
pub struct VertexAttribute<T: VertexFormat> {
    name: &'static str,
    id: OnceCell<VertexAttributeId>,
    _fmt: PhantomData<T>,
}

static VERTEX_ATTRIBUTE_INDEX_ALLOCATOR: AtomicUsize = AtomicUsize::new(0);

impl<T: VertexFormat> VertexAttribute<T> {
    /// Constructs a new vertex format struct
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            id: OnceCell::new(),
            _fmt: PhantomData,
        }
    }

    /// Gets the attribute identifier information
    pub fn id(&self) -> &VertexAttributeId {
        self.id.get_or_init(|| VertexAttributeId {
            inner: VERTEX_ATTRIBUTE_INDEX_ALLOCATOR.fetch_add(1, Ordering::Relaxed),
            name: self.name,
            meta: &T::META,
        })
    }

    /// Gets the name of the attribute
    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl<T: VertexFormat> Deref for VertexAttribute<T> {
    type Target = VertexAttributeId;

    fn deref(&self) -> &Self::Target {
        self.id()
    }
}

/// Predefined vertex attributes
pub mod attributes {
    use tempest_math::f32::{vec2::Vec2, vec3::Vec3};

    use super::VertexAttribute;

    /// Position vertex attribute
    pub static POSITION: VertexAttribute<Vec2> = VertexAttribute::new("position");

    /// UV coordinate 0 vertex attribute
    pub static UVCOORD_0: VertexAttribute<Vec3> = VertexAttribute::new("uvcoord_0");

    /// Normal vector vertex attribute
    pub static NORMAL: VertexAttribute<Vec3> = VertexAttribute::new("normal");

    /// Tangent vector vertex attribute
    pub static TANGENT: VertexAttribute<Vec3> = VertexAttribute::new("tangent");
}
