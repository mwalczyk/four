use std::fs::File;
use std::io::{BufRead, BufReader};
use std::mem;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr;

use cgmath::{self, ElementWise, InnerSpace, Vector3, Vector4, Zero};
use gl;
use gl::types::*;

use hyperplane::Hyperplane;
use rotations;
use tetrahedron::Tetrahedron;

pub struct Polytope {
    vertices: Vec<Vector4<f32>>,
    edges: Vec<u32>,
    faces: Vec<u32>,
    solids: Vec<u32>,
    vertices_per_edge: u32,
    edges_per_face: u32,
    faces_per_solid: u32,
    vao: u32,
    vbo: u32,
    ebo: u32,
}

impl Polytope {
    /// Loads the shape file at the specified `path`. The file
    /// should follow the format:
    ///
    /// ```
    /// number_of_vertices
    /// x0 y0 z0 w0
    /// x1 y1 z1 w1
    /// etc
    ///
    /// number_of_edges
    /// src0 dst0
    /// src1 dst1
    /// etc
    ///
    /// number_of_faces
    /// number_of_vertices_face0 f00 f01 f02 f03
    /// number_of_vertices_face1 f10 f11 f12 f13
    /// etc
    /// ```
    pub fn from_file(path: &Path) -> Polytope {
        let file = File::open(path).unwrap();
        let mut reader = BufReader::new(file);
        let mut number_of_entries = 0usize;
        let mut entry_count = String::new();

        // Load vertex data (4 entries per vertex).
        reader.read_line(&mut entry_count);
        number_of_entries = entry_count.trim().parse().unwrap();
        let mut vertices = Vec::with_capacity(number_of_entries * 4);

        for _ in 0..number_of_entries {
            let mut line = String::new();
            reader.read_line(&mut line);

            let mut all_coordinates = line.split_whitespace();
            let x = all_coordinates.next().unwrap().trim().parse().unwrap();
            let y = all_coordinates.next().unwrap().trim().parse().unwrap();
            let z = all_coordinates.next().unwrap().trim().parse().unwrap();
            let w = all_coordinates.next().unwrap().trim().parse().unwrap();

            vertices.push(Vector4::new(x, y, z, w));

            //            for entry in line.split_whitespace() {
            //                let data: f32 = entry.trim().parse().unwrap();
            //                vertices.push(data);
            //            }
        }
        entry_count.clear();

        // Load edge data (2 entries per edge).
        reader.read_line(&mut entry_count);
        number_of_entries = entry_count.trim().parse().unwrap();
        let mut edges = Vec::with_capacity(number_of_entries * 2);

        for _ in 0..number_of_entries {
            let mut line = String::new();
            reader.read_line(&mut line);

            for entry in line.split_whitespace() {
                let data: u32 = entry.trim().parse().unwrap();
                edges.push(data);
            }
        }
        entry_count.clear();

        // Load face data (4 entries per face).
        reader.read_line(&mut entry_count);
        number_of_entries = entry_count.trim().parse().unwrap();
        let mut faces = Vec::with_capacity(number_of_entries * 4);

        for _ in 0..number_of_entries {
            let mut line = String::new();
            reader.read_line(&mut line);

            for entry in line.split_whitespace() {
                let data: u32 = entry.trim().parse().unwrap();
                faces.push(data);
            }
        }
        entry_count.clear();

        // Load solid data (6 entries per solid).
        reader.read_line(&mut entry_count);
        number_of_entries = entry_count.trim().parse().unwrap();
        let mut solids = Vec::with_capacity(number_of_entries * 6);

        for _ in 0..number_of_entries {
            let mut line = String::new();
            reader.read_line(&mut line);

            for entry in line.split_whitespace() {
                let data: u32 = entry.trim().parse().unwrap();
                solids.push(data);
            }
        }

        println!(
            "Loaded file with {} vertices, {} edges, {} faces, and {} solids",
            vertices.len() / 4,
            edges.len() / 2,
            faces.len() / 4,
            solids.len() / 6
        );

        let mut polytope = Polytope {
            vertices,
            edges,
            faces,
            solids,
            vertices_per_edge: 2,
            edges_per_face: 4,
            faces_per_solid: 6,
            vao: 0,
            vbo: 0,
            ebo: 0,
        };

        polytope.init_render_objects();
        polytope
    }

    pub fn get_number_of_vertices(&self) -> usize {
        self.vertices.len()
    }

    pub fn get_number_of_edges(&self) -> usize {
        self.edges.len() / self.vertices_per_edge as usize
    }

    pub fn get_number_of_faces(&self) -> usize {
        self.faces.len() / self.edges_per_face as usize
    }

    /// Returns an unordered list of the unique vertices that make up the face at
    /// index `face`.
    pub fn get_vertices_for_face(&self, face: u32) -> Vec<Vector4<f32>> {
        let mut visited_vertices = Vec::new();
        let mut unique_vertices = Vec::new();

        let idx_face_s = (face * self.edges_per_face) as usize;
        let idx_face_e = (face * self.edges_per_face + self.edges_per_face) as usize;
        let edges = &self.faces[idx_face_s..idx_face_e];

        for edge in edges {
            let idx_edge_s = (*edge * self.vertices_per_edge) as usize;
            let idx_edge_e = (*edge * self.vertices_per_edge + self.vertices_per_edge) as usize;
            let pair = &self.edges[idx_edge_s..idx_edge_e];

            if !visited_vertices.contains(&pair[0]) {
                visited_vertices.push(pair[0]);
                unique_vertices.push(self.vertices[pair[0] as usize]);
            }
            if !visited_vertices.contains(&pair[1]) {
                visited_vertices.push(pair[1]);
                unique_vertices.push(self.vertices[pair[1] as usize]);
            }
        }

        unique_vertices
    }

    pub fn get_vertices_for_solids(&self) {
        // TODO
    }

    /// An H-representation of a polytope is a list of hyperplanes whose
    /// intersection produces the desired shape.
    ///
    /// Reference: `https://en.wikipedia.org/wiki/Convex_polytope#Intersection_of_half-spaces`
    /// See also: `facet enumeration`, `vertex enumeration`
    pub fn get_h_representation(&self) -> Vec<Hyperplane> {
        vec![
            Hyperplane::new(Vector4::unit_x(), 1.0),
            Hyperplane::new(Vector4::unit_x() * -1.0, 1.0),
            Hyperplane::new(Vector4::unit_y(), 1.0),
            Hyperplane::new(Vector4::unit_y() * -1.0, 1.0),
            Hyperplane::new(Vector4::unit_z(), 1.0),
            Hyperplane::new(Vector4::unit_z() * -1.0, 1.0),
            Hyperplane::new(Vector4::unit_w(), 1.0),
            Hyperplane::new(Vector4::unit_w() * -1.0, 1.0),
        ]
    }

    /// Given the H-representation of this polytope, return a list of lists, where
    /// each sub-list contains the indices of all faces that are inside of the `i`th
    /// hyerplane.
    pub fn gather_solids(&self) -> Vec<Vec<u32>> {
        let mut solids = Vec::new();
        let h_representation = self.get_h_representation();

        for hyperplane in h_representation.iter() {
            let mut faces_in_hyperplane = Vec::new();

            for face_index in 0..self.get_number_of_faces() {
                let face_vertices = self.get_vertices_for_face(face_index as u32);
                let mut inside = true;

                for vertex in face_vertices.iter() {
                    if !hyperplane.inside(&vertex) {
                        inside = false;
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

            solids.push(faces_in_hyperplane);
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

    fn palette(
        &self,
        t: f32,
        a: &Vector3<f32>,
        b: &Vector3<f32>,
        c: &Vector3<f32>,
        d: &Vector3<f32>,
    ) -> Vector3<f32> {
        use std::f32;

        // TODO: there should be a way to iterate over the `Vector4<f32>` and do this...
        let mut temp = (c * t + d) * 2.0 * f32::consts::PI;
        temp.x = temp.x.cos();
        temp.y = temp.y.cos();
        temp.z = temp.z.cos();

        a + b.mul_element_wise(temp)
    }

    pub fn tetrahedralize(&mut self, hyperplane: &Hyperplane) -> Vec<Tetrahedron> {
        let mut tetrahedrons = Vec::new();

        let get_color_for_tetrahedron = |t: f32| {
            self.palette(
                t,
                &Vector3::new(0.5, 0.5, 0.5),
                &Vector3::new(0.5, 0.5, 0.5),
                &Vector3::new(1.0, 1.0, 1.0),
                &Vector3::new(0.00, 0.33, 0.67),
            ).extend(1.0)
        };
        for (solid, faces) in self.solids
            .chunks(self.faces_per_solid as usize)
            .enumerate()
        {
            // The index of the vertex that all tetrahedrons making up this solid
            // will connect to.
            let mut apex = u32::max_value();

            // Iterate over each face of the current cell.
            for face in faces {
                // Retrieve the indices of all of the edges that make up this face.
                let idx_face_s = (*face * self.edges_per_face) as usize;
                let idx_face_e = (*face * self.edges_per_face + self.edges_per_face) as usize;
                let edges = &self.faces[idx_face_s..idx_face_e];

                // Retrieve the (unique) indices of all of the vertices that make up this face.
                let mut face_vertices = Vec::new();
                for edge in edges {
                    let idx_edge_s = (*edge * self.vertices_per_edge) as usize;
                    let idx_edge_e =
                        (*edge * self.vertices_per_edge + self.vertices_per_edge) as usize;
                    let pair = &self.edges[idx_edge_s..idx_edge_e];
                    let vertex_s = pair[0];
                    let vertex_e = pair[1];

                    // If the apex vertex has not been assigned yet, assign it to this face's
                    // first vertex and continue.
                    if apex == u32::max_value() {
                        apex = vertex_s;
                    }

                    if !face_vertices.contains(&vertex_s) {
                        face_vertices.push(vertex_s);
                    }

                    if !face_vertices.contains(&vertex_e) {
                        face_vertices.push(vertex_e);
                    }
                }

                // We only want to tetrahedralize faces that are NOT connected to the apex.
                if !face_vertices.contains(&apex) {
                    // First, we need to triangulate this face into two, non-overlapping
                    // triangles.
                    //
                    // a -- b
                    // |  / |
                    // | /  |
                    // c -- d
                    //
                    assert_eq!(face_vertices.len(), 4);

                    // Compute the face normal.
                    let v0 = self.vertices[face_vertices[0] as usize];
                    let v1 = self.vertices[face_vertices[1] as usize];
                    let v2 = self.vertices[face_vertices[2] as usize];
                    let edge_1_0 = v1 - v0;
                    let edge_2_0 = v2 - v0;
                    let edge_2_1 = v2 - v1;

                    let face_hyperplane =
                        Hyperplane::new(rotations::cross(&edge_1_0, &edge_2_0, &edge_2_1), 0.0);

                    for i in 0..(self.vertices.len() / 4) {}

                    // Collect all 4D vertices and sort.
                    let face_vertices_sorted = rotations::sort_points_on_plane(
                        &face_vertices
                            .iter()
                            .map(|index| self.vertices[*index as usize])
                            .collect::<Vec<_>>(),
                        &face_hyperplane,
                    );

                    // Create a triangle fan, starting at the first vertex of the sorted list.
                    // Connect each resulting triangle to the apex vertex to create a full
                    // tetrahedron.
                    for i in 1..face_vertices_sorted.len() - 1 {
                        tetrahedrons.push(Tetrahedron::new(
                            [
                                face_vertices_sorted[0],
                                face_vertices_sorted[i + 0],
                                face_vertices_sorted[i + 1],
                                self.vertices[apex as usize],
                            ],
                            get_color_for_tetrahedron(solid as f32 / 8.0),
                        ));
                    }
                }
            }
        }

        tetrahedrons
    }
}
