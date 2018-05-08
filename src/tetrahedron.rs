use cgmath::{self, Vector4};

pub trait Tetrahedralize {
    fn generate() -> Vec<Tetrahedron>;
}

/// A struct representing a tetrahedron (3-simplex) embedded in 4-dimensions. This
/// is the building block for all 4-dimensional meshes in the `four` renderer.
pub struct Tetrahedron {
    pub vertices: [Vector4<f32>; 4],
    pub cell_index: u32,
    pub cell_centroid: Vector4<f32>,
}

impl Tetrahedron {
    /// Create a new tetrahedron from an array of 4 vertices embedded in a 4-dimensional
    /// space.
    pub fn new(
        vertices: [Vector4<f32>; 4],
        cell_index: u32,
        cell_centroid: Vector4<f32>,
    ) -> Tetrahedron {
        Tetrahedron {
            vertices,
            cell_index,
            cell_centroid,
        }
    }

    /// Note that OpenGL expects these to be `u32`s.
    pub fn get_edge_indices() -> [(u32, u32); 6] {
        [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]
    }

    /// Returns the indices for a quadrilateral slice, which is constructed from two
    /// triangles.
    pub fn get_quad_indices() -> [(u32, u32, u32); 2] {
        [(0, 1, 2), (0, 2, 3)]
    }
}
