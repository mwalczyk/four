use std::f32;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::mem;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr;

use cgmath::{self, Array, InnerSpace, Vector3, Vector4, Zero};
use gl;
use gl::types::*;

use hyperplane::Hyperplane;
use polychora::{Definition, Polychoron};
use rotations::{self, Plane};
use tetrahedron::Tetrahedron;
use utilities;

/// A 4-dimensional mesh.
pub struct Mesh {
    pub vertices: Vec<Vector4<f32>>,
    pub edges: Vec<u32>,
    pub faces: Vec<u32>,
    pub def: Definition,
    vao: u32,
    vbo: u32,
    ebo: u32,
}

impl Mesh {
    pub fn new(polychoron: Polychoron) -> Mesh {
        let mut mesh = Mesh {
            vertices: polychoron.get_vertices(),
            edges: polychoron.get_edges(),
            faces: polychoron.get_faces(),
            def: polychoron.get_definition(),
            vao: 0,
            vbo: 0,
            ebo: 0,
        };

        mesh.init_render_objects();
        mesh
    }

    /// Returns the number of unique vertices in this mesh.
    pub fn get_number_of_vertices(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the number of unique edges in this mesh.
    pub fn get_number_of_edges(&self) -> usize {
        self.edges.len() / self.def.vertices_per_edge as usize
    }

    /// Returns the number of unique faces in this mesh.
    pub fn get_number_of_faces(&self) -> usize {
        self.faces.len() / self.def.vertices_per_face as usize
    }

    /// Returns the `i`th vertex of this polytope.
    pub fn get_vertex(&self, i: u32) -> Vector4<f32> {
        self.vertices[i as usize]
    }

    /// Returns an unordered tuple of the two vertices that make up the `i`th
    /// edge of this polytope.
    pub fn get_vertices_for_edge(&self, i: u32) -> (Vector4<f32>, Vector4<f32>) {
        let idx_edge_s = (i * self.def.vertices_per_edge) as usize;
        let idx_edge_e = (i * self.def.vertices_per_edge + self.def.vertices_per_edge) as usize;
        let pair = &self.edges[idx_edge_s..idx_edge_e];

        (self.get_vertex(pair[0]), self.get_vertex(pair[1]))
    }

    /// Returns an unordered list of the unique vertices that make up the `i`th
    /// face of this polytope.
    pub fn get_vertices_for_face(&self, i: u32) -> Vec<Vector4<f32>> {
        let idx_face_s = (i * self.def.vertices_per_face) as usize;
        let idx_face_e = (i * self.def.vertices_per_face + self.def.vertices_per_face) as usize;
        let vertex_ids = &self.faces[idx_face_s..idx_face_e];

        let vertices = vertex_ids
            .iter()
            .map(|id| self.get_vertex(*id))
            .collect::<Vec<_>>();

        vertices
    }

    /// The H-representation of a convex polytope is the list of hyperplanes whose
    /// intersection produces the desired shape. Together, these hyperplanes form
    /// a "boundary" for the polytope. We use this representation in order to determine
    /// which faces belong to each of the cells that form the polytope's surface.
    ///
    /// See: `https://en.wikipedia.org/wiki/Convex_polytope#Intersection_of_half-spaces`
    pub fn get_h_representation(&self) -> Vec<Hyperplane> {
        // The circumradius of the 120-cell is: 2√2, ~2.828

        // The inner "radius" of this particular 120-cell is: -2 * φ^2
        let golden_ratio: f32 = (1.0 + 5.0f32.sqrt()) / 2.0;
        let displacement = -golden_ratio.powf(2.0) * 2.0;
        let d2 = -8.47213;

        let representation = vec![
            Hyperplane::new(Vector4::new(2.0, 0.0, 0.0, 0.0), displacement),
            Hyperplane::new(Vector4::new(-2.0, 0.0, 0.0, 0.0), displacement),
            Hyperplane::new(Vector4::new(0.0, 2.0, 0.0, 0.0), displacement),
            Hyperplane::new(Vector4::new(0.0, -2.0, 0.0, 0.0), displacement),
            Hyperplane::new(Vector4::new(0.0, 0.0, 2.0, 0.0), displacement),
            Hyperplane::new(Vector4::new(0.0, 0.0, -2.0, 0.0), displacement),
            Hyperplane::new(Vector4::new(0.0, 0.0, 0.0, 2.0), displacement),
            Hyperplane::new(Vector4::new(0.0, 0.0, 0.0, -2.0), displacement),
            Hyperplane::new(Vector4::new(2.61803, 1.0, 0.0, -1.61803), d2),
            Hyperplane::new(Vector4::new(2.61803, 0.0, 1.61803, -1.0), d2),
            Hyperplane::new(Vector4::new(2.61803, 0.0, -1.61803, -1.0), d2),
            Hyperplane::new(Vector4::new(2.61803, -1.0, 0.0, -1.61803), d2),
            Hyperplane::new(Vector4::new(2.61803, -1.61803, -1.0, 0.0), d2),
            Hyperplane::new(Vector4::new(1.61803, 2.61803, 0.0, -1.0), d2),
            Hyperplane::new(Vector4::new(1.61803, 1.0, -2.61803, 0.0), d2),
            Hyperplane::new(Vector4::new(1.61803, 0.0, 1.0, -2.61803), d2),
            Hyperplane::new(Vector4::new(1.61803, 0.0, -1.0, -2.61803), d2),
            Hyperplane::new(Vector4::new(1.61803, -1.0, -2.61803, 0.0), d2),
            Hyperplane::new(Vector4::new(1.61803, -2.61803, 0.0, -1.0), d2),
            Hyperplane::new(Vector4::new(1.0, 2.61803, -1.61803, 0.0), d2),
            Hyperplane::new(Vector4::new(1.0, 1.61803, 0.0, -2.61803), d2),
            Hyperplane::new(Vector4::new(1.0, 0.0, 2.61803, -1.61803), d2),
            Hyperplane::new(Vector4::new(1.0, 0.0, -2.61803, -1.61803), d2),
            Hyperplane::new(Vector4::new(1.0, -1.61803, 0.0, -2.61803), d2),
            Hyperplane::new(Vector4::new(0.0, 2.61803, 1.0, -1.61803), d2),
            Hyperplane::new(Vector4::new(0.0, 2.61803, -1.0, -1.61803), d2),
            Hyperplane::new(Vector4::new(0.0, 1.61803, 2.61803, -1.0), d2),
            Hyperplane::new(Vector4::new(0.0, 1.61803, -2.61803, -1.0), d2),
            Hyperplane::new(Vector4::new(0.0, 1.0, 1.61803, -2.61803), d2),
            Hyperplane::new(Vector4::new(0.0, 1.0, -1.61803, -2.61803), d2),
            Hyperplane::new(Vector4::new(0.0, -1.0, 1.61803, -2.61803), d2),
            Hyperplane::new(Vector4::new(0.0, -1.0, -1.61803, -2.61803), d2),
            Hyperplane::new(Vector4::new(0.0, -1.61803, 2.61803, -1.0), d2),
            Hyperplane::new(Vector4::new(0.0, -1.61803, -2.61803, -1.0), d2),
            Hyperplane::new(Vector4::new(0.0, -2.61803, 1.0, -1.61803), d2),
            Hyperplane::new(Vector4::new(0.0, -2.61803, -1.0, -1.61803), d2),
            Hyperplane::new(Vector4::new(-1.0, 1.61803, 0.0, 2.61803), d2),
            Hyperplane::new(Vector4::new(-1.0, 0.0, 2.61803, -1.61803), d2),
            Hyperplane::new(Vector4::new(-1.0, 0.0, -2.61803, -1.61803), d2),
            Hyperplane::new(Vector4::new(-1.0, -1.61803, 0.0, -2.61803), d2),
            Hyperplane::new(Vector4::new(-1.0, -2.61803, -1.61803, 0.0), d2),
            Hyperplane::new(Vector4::new(-1.61803, 2.61803, 0.0, -1.0), d2),
            Hyperplane::new(Vector4::new(-1.61803, 1.0, -2.61803, 0.0), d2),
            Hyperplane::new(Vector4::new(-1.61803, 0.0, 1.0, -2.61803), d2),
            Hyperplane::new(Vector4::new(-1.61803, 0.0, -1.0, -2.61803), d2),
            Hyperplane::new(Vector4::new(-1.61803, -1.0, -2.61803, 0.0), d2),
            Hyperplane::new(Vector4::new(-1.61803, -2.61803, 0.0, -1.0), d2),
            Hyperplane::new(Vector4::new(-2.61803, 1.61803, -1.0, 0.0), d2),
            Hyperplane::new(Vector4::new(-2.61803, 1.0, 0.0, -1.61803), d2),
            Hyperplane::new(Vector4::new(-2.61803, 0.0, 1.61803, -1.0), d2),
            Hyperplane::new(Vector4::new(-2.61803, 0.0, -1.61803, -1.0), d2),
            Hyperplane::new(Vector4::new(-2.61803, -1.0, -0.0, -1.61803), d2),
            Hyperplane::new(Vector4::new(-2.61803, -1.61803, -1.0, -0.0), d2),
            Hyperplane::new(Vector4::new(-2.61803, -1.61803, 1.0, -0.0), d2),
            Hyperplane::new(Vector4::new(-2.61803, -1.0, 0.0, 1.61803), d2),
            Hyperplane::new(Vector4::new(-2.61803, 0.0, -1.61803, 1.0), d2),
            Hyperplane::new(Vector4::new(-2.61803, 0.0, 1.61803, 1.0), d2),
            Hyperplane::new(Vector4::new(-2.61803, 1.0, 0.0, 1.61803), d2),
            Hyperplane::new(Vector4::new(-2.61803, 1.61803, 1.0, 0.0), d2),
            Hyperplane::new(Vector4::new(-1.61803, -2.61803, 0.0, 1.0), d2),
            Hyperplane::new(Vector4::new(-1.61803, -1.0, 2.61803, 0.0), d2),
            Hyperplane::new(Vector4::new(-1.61803, -0.0, -1.0, 2.61803), d2),
            Hyperplane::new(Vector4::new(-1.61803, 0.0, 1.0, 2.61803), d2),
            Hyperplane::new(Vector4::new(-1.61803, 1.0, 2.61803, 0.0), d2),
            Hyperplane::new(Vector4::new(-1.61803, 2.61803, 0.0, 1.0), d2),
            Hyperplane::new(Vector4::new(-1.0, -2.61803, 1.61803, 0.0), d2),
            Hyperplane::new(Vector4::new(-1.0, -1.61803, 0.0, 2.61803), d2),
            Hyperplane::new(Vector4::new(-1.0, 0.0, -2.61803, 1.61803), d2),
            Hyperplane::new(Vector4::new(-1.0, 0.0, 2.61803, 1.61803), d2),
            Hyperplane::new(Vector4::new(-1.0, 1.61803, 0.0, -2.61803), d2),
            Hyperplane::new(Vector4::new(-1.0, 2.61803, -1.61803, 0.0), d2),
            Hyperplane::new(Vector4::new(-1.0, 2.61803, 1.61803, 0.0), d2),
            Hyperplane::new(Vector4::new(0.0, -2.61803, -1.0, 1.61803), d2),
            Hyperplane::new(Vector4::new(0.0, -2.61803, 1.0, 1.61803), d2),
            Hyperplane::new(Vector4::new(0.0, -1.61803, -2.61803, 1.0), d2),
            Hyperplane::new(Vector4::new(0.0, -1.61803, 2.61803, 1.0), d2),
            Hyperplane::new(Vector4::new(0.0, -1.0, -1.61803, 2.61803), d2),
            Hyperplane::new(Vector4::new(0.0, -1.0, 1.61803, 2.61803), d2),
            Hyperplane::new(Vector4::new(0.0, 1.0, -1.61803, 2.61803), d2),
            Hyperplane::new(Vector4::new(0.0, 1.0, 1.61803, 2.61803), d2),
            Hyperplane::new(Vector4::new(0.0, 1.61803, -2.61803, 1.0), d2),
            Hyperplane::new(Vector4::new(0.0, 1.61803, 2.61803, 1.0), d2),
            Hyperplane::new(Vector4::new(0.0, 2.61803, -1.0, 1.61803), d2),
            Hyperplane::new(Vector4::new(0.0, 2.61803, 1.0, 1.61803), d2),
            Hyperplane::new(Vector4::new(1.0, -2.61803, -1.61803, 0.0), d2),
            Hyperplane::new(Vector4::new(1.0, -2.61803, 1.61803, 0.0), d2),
            Hyperplane::new(Vector4::new(1.0, -1.61803, 0.0, 2.61803), d2),
            Hyperplane::new(Vector4::new(1.0, 0.0, -2.61803, 1.61803), d2),
            Hyperplane::new(Vector4::new(1.0, 0.0, 2.61803, 1.61803), d2),
            Hyperplane::new(Vector4::new(1.0, 1.61803, 0.0, 2.61803), d2),
            Hyperplane::new(Vector4::new(1.0, 2.61803, 1.61803, 0.0), d2),
            Hyperplane::new(Vector4::new(1.61803, -2.61803, 0.0, 1.0), d2),
            Hyperplane::new(Vector4::new(1.61803, -1.0, 2.61803, 0.0), d2),
            Hyperplane::new(Vector4::new(1.61803, 0.0, -1.0, 2.61803), d2),
            Hyperplane::new(Vector4::new(1.61803, 0.0, 1.0, 2.61803), d2),
            Hyperplane::new(Vector4::new(1.61803, 1.0, 2.61803, 0.0), d2),
            Hyperplane::new(Vector4::new(1.61803, 2.61803, 0.0, 1.0), d2),
            Hyperplane::new(Vector4::new(2.61803, -1.61803, 1.0, 0.0), d2),
            Hyperplane::new(Vector4::new(2.61803, -1.0, 0.0, 1.61803), d2),
            Hyperplane::new(Vector4::new(2.61803, 0.0, -1.61803, 1.0), d2),
            Hyperplane::new(Vector4::new(2.61803, 0.0, 1.61803, 1.0), d2),
            Hyperplane::new(Vector4::new(2.61803, 1.0, 0.0, 1.61803), d2),
            Hyperplane::new(Vector4::new(2.61803, 1.61803, -1.0, 0.0), d2),
            Hyperplane::new(Vector4::new(2.61803, 1.61803, 1.0, 0.0), d2),
            Hyperplane::new(Vector4::new(1.0, 1.0, 1.0, 1.0), displacement),
            Hyperplane::new(Vector4::new(-1.0, 1.0, 1.0, 1.0), displacement),
            Hyperplane::new(Vector4::new(1.0, -1.0, 1.0, 1.0), displacement),
            Hyperplane::new(Vector4::new(1.0, 1.0, -1.0, 1.0), displacement),
            Hyperplane::new(Vector4::new(1.0, 1.0, 1.0, -1.0), displacement),
            Hyperplane::new(Vector4::new(-1.0, -1.0, 1.0, 1.0), displacement),
            Hyperplane::new(Vector4::new(-1.0, 1.0, -1.0, 1.0), displacement),
            Hyperplane::new(Vector4::new(-1.0, 1.0, 1.0, -1.0), displacement),
            Hyperplane::new(Vector4::new(1.0, -1.0, -1.0, 1.0), displacement),
            Hyperplane::new(Vector4::new(1.0, -1.0, 1.0, -1.0), displacement),
            Hyperplane::new(Vector4::new(1.0, 1.0, -1.0, -1.0), displacement),
            Hyperplane::new(Vector4::new(-1.0, -1.0, -1.0, 1.0), displacement),
            Hyperplane::new(Vector4::new(-1.0, -1.0, 1.0, -1.0), displacement),
            Hyperplane::new(Vector4::new(-1.0, 1.0, -1.0, -1.0), displacement),
            Hyperplane::new(Vector4::new(1.0, -1.0, -1.0, -1.0), displacement),
            Hyperplane::new(Vector4::new(-1.0, -1.0, -1.0, -1.0), displacement),
        ];

        println!(
            "Number of hyperplanes in 120-cell's H-representation: {}",
            representation.len()
        );

        let hypercube = false;
        if hypercube {
            return vec![
                Hyperplane::new(Vector4::unit_x(), 1.0),
                Hyperplane::new(Vector4::unit_x() * -1.0, 1.0),
                Hyperplane::new(Vector4::unit_y(), 1.0),
                Hyperplane::new(Vector4::unit_y() * -1.0, 1.0),
                Hyperplane::new(Vector4::unit_z(), 1.0),
                Hyperplane::new(Vector4::unit_z() * -1.0, 1.0),
                Hyperplane::new(Vector4::unit_w(), 1.0),
                Hyperplane::new(Vector4::unit_w() * -1.0, 1.0),
            ];
        } else {
            return representation;
        }
    }

    /// The V-representation of a convex polytope is simply the list of vertices,
    /// which form the convex hull of the volume spanned by the polytope.
    ///
    /// See: `https://en.wikipedia.org/wiki/Convex_polytope#Vertex_representation_(convex_hull)`
    pub fn get_v_representation(&self) -> &Vec<Vector4<f32>> {
        &self.vertices
    }

    /// Given the H-representation of this polytope, return a list of lists, where
    /// each sub-list contains the indices of all faces that are inside of the `i`th
    /// hyperplane.
    pub fn gather_cells(&self) -> Vec<(Hyperplane, Vec<u32>)> {
        let mut solids = Vec::new();

        for hyperplane in self.get_h_representation().iter() {
            let mut faces_in_hyperplane = Vec::new();

            // Iterate over all of the faces of this polytope. For the 120-cell, for example,
            // there are 720 faces, each of which has 5 vertices associated with it.
            for face_index in 0..self.get_number_of_faces() {
                let face_vertices = self.get_vertices_for_face(face_index as u32);

                assert_eq!(face_vertices.len(), self.def.vertices_per_face as usize);

                // Check if all of the vertices of this face are inside the bounding hyperplane.
                let mut inside = true;
                for vertex in face_vertices.iter() {
                    if !hyperplane.inside(&vertex) {
                        inside = false;
                        break;
                    }
                }
                if inside {
                    faces_in_hyperplane.push(face_index as u32);
                }
            }

            println!(
                "{} faces found for hyperplane with normal {:?}",
                faces_in_hyperplane.len(),
                hyperplane.normal
            );

            solids.push((*hyperplane, faces_in_hyperplane));
        }

        solids
    }

    pub fn draw(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);

            gl::DrawElements(
                gl::LINES,
                self.edges.len() as i32,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
        }
    }

    fn init_render_objects(&mut self) {
        unsafe {
            gl::CreateVertexArrays(1, &mut self.vao);

            let mut size = (self.vertices.len() * mem::size_of::<Vector4<f32>>()) as GLsizeiptr;
            gl::CreateBuffers(1, &mut self.vbo);
            gl::NamedBufferData(
                self.vbo,
                size,
                self.vertices.as_ptr() as *const GLvoid,
                gl::DYNAMIC_DRAW,
            );

            size = (self.edges.len() * mem::size_of::<u32>()) as GLsizeiptr;
            gl::CreateBuffers(1, &mut self.ebo);
            gl::NamedBufferData(
                self.ebo,
                size,
                self.edges.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            let binding = 0;
            gl::EnableVertexArrayAttrib(self.vao, 0);
            gl::VertexArrayAttribFormat(self.vao, 0, 4, gl::FLOAT, gl::FALSE, 0);
            gl::VertexArrayAttribBinding(self.vao, 0, binding);
            gl::VertexArrayElementBuffer(self.vao, self.ebo);

            let offset = 0;
            gl::VertexArrayVertexBuffer(
                self.vao,
                binding,
                self.vbo,
                offset,
                (mem::size_of::<Vector4<f32>>() as usize) as i32,
            );
        }
    }

    /// Performs of a tetrahedral decomposition of the polytope.
    pub fn tetrahedralize(&mut self) -> Vec<Tetrahedron> {
        let mut tetrahedrons = Vec::new();
        let simple = true;

        for (cell_index, plane_and_faces) in self.gather_cells().iter().enumerate() {
            // The vertex that all tetrahedrons making up this solid will connect to.
            let mut apex = Vector4::from_value(f32::MAX);

            let (hyperplane, faces) = plane_and_faces;
            let mut prev_len = tetrahedrons.len();

            // Calculate the centroid of this cell.
            let mut cell_centroid = Vector4::from_value(0.0);
            for index in faces.iter() {
                let face_vertices = self.get_vertices_for_face(*index);
                let face_centroid = face_vertices.iter().sum::<Vector4<f32>>();
                cell_centroid += face_centroid;
            }
            cell_centroid /= (self.def.vertices_per_face * self.def.faces_per_cell) as f32;

            // Iterate over each face of the current cell.
            for face in faces {
                let face_vertices = self.get_vertices_for_face(*face);
                // First, we need to triangulate this face into two, non-overlapping
                // triangles.
                //
                // a -- b
                // |  / |
                // | /  |
                // c -- d
                //
                // Collect all 4D vertices and sort.
                let face_vertices_sorted =
                    rotations::sort_points_on_plane(&face_vertices, &hyperplane);

                if simple {
                    if apex.x == f32::MAX {
                        apex = face_vertices[0];
                    }

                    // We only want to tetrahedralize faces that are NOT connected to the apex.
                    if !face_vertices.contains(&apex) {
                        // Create a triangle fan, starting at the first vertex of the sorted list.
                        // Connect each resulting triangle to the apex vertex to create a full
                        // tetrahedron.
                        for i in 1..face_vertices_sorted.len() - 1 {
                            tetrahedrons.push(Tetrahedron::new(
                                [
                                    face_vertices_sorted[0],
                                    face_vertices_sorted[i + 0],
                                    face_vertices_sorted[i + 1],
                                    apex,
                                ],
                                utilities::from_hex(0xffffff, 1.0),
                                cell_index as u32,
                                cell_centroid,
                            ));
                        }
                    }
                } else {

                    //                    let face_centroid = face_vertices.iter().sum::<Vector4<f32>>();
                    //                    let original_radius = (cell_centroid - face_centroid).magnitude();
                    //                    println!("original radius: {}", original_radius);
                    //
                    //                    let thickness = 0.75;
                    //
                    //                    // Iterate over all of the triangles that make up this face.
                    //                    for index in 0..face_vertices_sorted.len()-1 {
                    //                        let mut new_faces = Vec::new();
                    //                        let mut new_tris = Vec::new();
                    //
                    //                        // Grab the first triangle that makes up this face.
                    //                        let triangle_vertices = vec![
                    //                            face_vertices_sorted[0],
                    //                            face_vertices_sorted[index + 0],
                    //                            face_vertices_sorted[index + 1],
                    //                        ];
                    //                        let triangle_centroid =
                    //                            triangle_vertices.iter().sum::<Vector4<f32>>() / 3.0;
                    //
                    //                        // Assign the apex to the first vertex of this triangle.
                    //                        let apex = triangle_vertices[0];
                    //
                    //                        // Now, create a scaled copy of the original face.
                    //                        let triangle_small = triangle_vertices
                    //                            .iter()
                    //                            .map(|vertex| {
                    //                                let mut vertex_small = *vertex;
                    //                                vertex_small -= triangle_centroid;
                    //                                vertex_small *= thickness;
                    //                                vertex_small += triangle_centroid;
                    //
                    //                                // Choose the translation amount based on the `thickness` variable.
                    //                                let amount = original_radius * (1.0 - thickness);
                    //                                let translation = (cell_centroid - triangle_centroid).normalize();
                    //
                    //                                // Move towards the cell center.
                    //                                vertex_small + translation * amount
                    //                            })
                    //                            .collect::<Vec<_>>();
                    //
                    //                        for i in 0..3 {
                    //                            let src = i;
                    //                            let dst = (i + 1) % 3;
                    //
                    //                            new_faces.push(vec![
                    //                                triangle_vertices[src],
                    //                                triangle_vertices[dst],
                    //                                triangle_small[src],
                    //                                triangle_small[dst],
                    //                            ]);
                    //
                    //                            new_tris.push((
                    //                                (
                    //                                    triangle_vertices[src],
                    //                                    triangle_vertices[dst],
                    //                                    triangle_small[src],
                    //                                ),
                    //                                (
                    //                                    triangle_small[src],
                    //                                    triangle_small[dst],
                    //                                    triangle_vertices[dst],
                    //                                ),
                    //                            ));
                    //                        }
                    //
                    //                        new_faces.push(vec![
                    //                            triangle_small[0],
                    //                            triangle_small[1],
                    //                            triangle_small[2]
                    //                        ]);
                    //
                    //                        new_tris.push((
                    //                            (triangle_small[0], triangle_small[1], triangle_small[2]),
                    //                            (triangle_small[0], triangle_small[1], triangle_small[2]),
                    //                        ));
                    //
                    //                        for (i, face) in new_faces.iter().enumerate() {
                    //                            if !face.contains(&apex) {
                    //                                // Add the first tetrahedron.
                    //                                tetrahedrons.push(Tetrahedron::new(
                    //                                    [(new_tris[i].0).0, (new_tris[i].0).1, (new_tris[i].0).2, apex],
                    //                                    utilities::from_hex(0xffffff, 1.0),
                    //                                    cell_index as u32,
                    //                                    cell_centroid,
                    //                                ));
                    //
                    //                                // Add the second tetrahedron.
                    //                                tetrahedrons.push(Tetrahedron::new(
                    //                                    [(new_tris[i].1).0, (new_tris[i].1).1, (new_tris[i].1).2, apex],
                    //                                    utilities::from_hex(0xffffff, 1.0),
                    //                                    cell_index as u32,
                    //                                    cell_centroid,
                    //                                ));
                    //                            }
                    //                        }
                    //                    }
                    //
                    //                    for i in 1..face_vertices_sorted.len() - 1 {
                    //                        tetrahedrons.push(Tetrahedron::new(
                    //                            [
                    //                                face_vertices_sorted[0],
                    //                                face_vertices_sorted[i + 0],
                    //                                face_vertices_sorted[i + 1],
                    //                                apex,
                    //                            ],
                    //                            utilities::from_hex(0xffffff, 1.0),
                    //                            cell_index as u32,
                    //                            cell_centroid,
                    //                        ));
                    //                    }
                }
            }

            println!(
                "{} tetrahedrons found for solid {}",
                tetrahedrons.len() - prev_len,
                cell_index
            );
        }

        tetrahedrons
    }
}