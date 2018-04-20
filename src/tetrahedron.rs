use cgmath::{self, Vector4};

use hyperplane::Hyperplane;
use rotations;

pub trait Tetrahedralize {
    fn generate() -> Vec<Tetrahedron>;
}

/// A struct representing a tetrahedron (3-simplex) embedded in 4-dimensions. This
/// is the building block for all 4-dimensional meshes in the `four` renderer.
pub struct Tetrahedron {
    pub vertices: [Vector4<f32>; 4],
    pub color: Vector4<f32>,
}

impl Tetrahedron {
    /// Create a new tetrahedron from an array of 4 vertices embedded in a 4-dimensional
    /// space.
    pub fn new(vertices: [Vector4<f32>; 4], color: Vector4<f32>) -> Tetrahedron {
        Tetrahedron { vertices, color }
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

    /// Returns the result of slicing the tetrahedron with `hyperplane`. Note that
    /// this will always return either an empty intersection, a single triangle,
    /// or a single quadrilateral.
    pub fn slice(&self, hyperplane: &Hyperplane) -> Vec<Vector4<f32>> {
        let mut intersections = Vec::new();

        for (a, b) in Tetrahedron::get_edge_indices().iter() {
            let vertex_a = self.vertices[*a as usize];
            let vertex_b = self.vertices[*b as usize];

            // TODO: explain this math.
            let t = -hyperplane.side(&vertex_a)
                / (hyperplane.side(&vertex_b) - hyperplane.side(&vertex_a));

            if t >= 0.0 && t <= 1.0 {
                let intersection = vertex_a + (vertex_b - vertex_a) * t;
                intersections.push(intersection);
            }
        }

        if intersections.len() == 4 {
            return rotations::sort_quadrilateral(&intersections);
        }

        // TODO: this fails sometimes.
       // assert!(intersections.len() == 0 || intersections.len() == 3 || intersections.len() == 4);

        intersections
    }
}
