use std::fs::File;
use std::io::{BufRead, BufReader};
use std::mem;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr;

use cgmath::{self, Vector3, Vector4, ElementWise, InnerSpace, Zero};
use gl;
use gl::types::*;

use hyperplane::Hyperplane;
use rotations;
use tetrahedron::Tetrahedron;

pub struct Polytope {
    vertices: Vec<f32>,
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

            for entry in line.split_whitespace() {
                let data: f32 = entry.trim().parse().unwrap();
                vertices.push(data);
            }
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

    /// Returns the vertex at `index`, as a single `Vector4`.
    pub fn get_vertex(&self, index: usize) -> Vector4<f32> {
        Vector4::new(
            self.vertices[index * 4 + 0],
            self.vertices[index * 4 + 1],
            self.vertices[index * 4 + 2],
            self.vertices[index * 4 + 3],
        )
    }

    pub fn get_vertices_for_faces(&self) {
        // TODO
    }

    pub fn get_vertices_for_solids(&self) {
        // TODO
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

            let mut size = (self.vertices.len() * mem::size_of::<f32>()) as GLsizeiptr;
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
                (mem::size_of::<f32>() * 4 as usize) as i32,
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
            self.palette(t,
                &Vector3::new(0.5, 0.5, 0.5),
                &Vector3::new(0.5, 0.5, 0.5),
                &Vector3::new(1.0, 1.0, 1.0),
                &Vector3::new(0.00, 0.33, 0.67),
            ).extend(1.0)
        };
        for (solid, faces) in self.solids.chunks(self.faces_per_solid as usize).enumerate() {
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

                    // Collect all 4D vertices and sort.
                    let quad_sorted = rotations::sort_quadrilateral(&face_vertices.iter().map(|index| {
                        self.get_vertex(*index as usize)
                    }).collect::<Vec<_>>(), hyperplane);

                    for (a, b, c) in Tetrahedron::get_quad_indices().iter() {
                        // Next, form a tetrahedron with each triangle and the apex vertex.
                        tetrahedrons.push(Tetrahedron::new(
                            [
                                quad_sorted[*a as usize],
                                quad_sorted[*b as usize],
                                quad_sorted[*c as usize],
                                self.get_vertex(apex as usize),
                            ],
                            get_color_for_tetrahedron(solid as f32 / 8.0)
                        ));
                    }
                }
            }
        }

        tetrahedrons
    }
}
