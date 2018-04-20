use cgmath::{self, Vector4};

use hyperplane::Hyperplane;

pub trait Tetrahedralize {
    fn generate() -> Vec<Tetrahedron>;
}

pub enum TetrahedronSlice {
    Empty,
    Triangle(Vec<Vector4<f32>>),
    Quadrilateral(Vec<Vector4<f32>>),
}

/// Note that OpenGL expects these to be `u32`s.
pub const TETRAHEDRON_INDICES: [(u32, u32); 6] = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];

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

    /// Returns the result of slicing the tetrahedron with `hyperplane`. Note that
    /// this will always return either an empty intersection, a single triangle,
    /// or a single quadrilateral.
    pub fn slice(&self, hyperplane: &Hyperplane) -> Vec<Vector4<f32>> {
        let mut intersections = Vec::new();

        for (a, b) in TETRAHEDRON_INDICES.iter() {
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
            // TODO
        }

        // TODO: this fails on start.
        //assert!(intersections.len() == 0 || intersections.len() == 3 || intersections.len() == 4);
        intersections
    }

    /// Given a set of four vertices embedded in 4-dimensions, find a proper ordering
    /// of `points[0]`, `points[1]`, `points[2]`, and `points[3]` such that the resulting
    /// list of vertices can be drawn as two distinct triangles.
    pub fn sort_quadrilateral(&self, points: &mut Vec<Vector4<f32>>) {
        assert_eq!(points.len(), 4);

        // TODO
    }
}
