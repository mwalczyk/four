use cgmath::{self, Vector4};

pub trait Tetrahedralize {
    fn generate() -> Vec<Tetrahedron>;
}

/// A struct representing a tetrahedron (3-simplex) embedded in 4-dimensions. This
/// is the building block for all 4-dimensional meshes in the `four` renderer.
pub struct Tetrahedron {
    /// The 4 vertices that make up this tetrahedron
    vertices: [Vector4<f32>; 4],

    /// The integer index of the cell that this tetrahedron belongs to
    pub cell_index: u32,

    /// The centroid (in 4-space) of the cell that this tetrahedron belongs to
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

    /// Returns an array of this tetrahedron's vertices.
    pub fn get_vertices(&self) -> &[Vector4<f32>; 4] {
        &self.vertices
    }

    /// Returns the integer index of the cell that this tetrahedon belongs to.
    pub fn get_cell_index(&self) -> u32 {
        self.cell_index
    }

    /// Returns the centroid (in 4-space) of the cell that this tetrahedron belongs to.
    pub fn get_cell_centroid(&self) -> Vector4<f32> {
        self.cell_centroid
    }

    /// Note that OpenGL expects these to be `u32`s.
    pub fn get_edge_indices() -> [(u32, u32); 6] {
        [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]
    }

    /// Returns the number of vertices that make up a tetrahedron.
    pub fn get_number_of_vertices() -> usize { 4 }

    /// Returns the number of edges that make up a tetrahedron.
    pub fn get_number_of_edges() -> usize {
        6
    }

    /// Slicing a tetrahedron with a plane will produce either 0, 3, or 4
    /// points of intersection. In the case that the slicing procedure returns
    /// 4 unique vertices, we need to know how to connect these vertices to
    /// form a closed polygon (i.e. a quadrilateral). Assuming these vertices
    /// are sorted in either a clockwise or counter-clockwise order (relative
    /// to the normal of the plane of intersection), we can use the indices
    /// here to produce two, non-overlapping triangles.
    ///
    /// Returns the indices for a quadrilateral slice.
    pub fn get_quad_indices() -> [(u32, u32, u32); 2] {
        [(0, 1, 2), (0, 2, 3)]
    }
}
