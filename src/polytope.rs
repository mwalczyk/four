use std::fs::File;
use std::io::{BufRead, BufReader};
use std::mem;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr;

use std::collections::HashSet;

use cgmath::{self, InnerSpace, Matrix4, Vector3, Vector4, Zero};
use gl;
use gl::types::*;

use hyperplane::Hyperplane;
use rotations;
use slice::Slice;
use tetrahedron::{Tetrahedron, TetrahedronSlice};

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
            gl::Enable(gl::VERTEX_PROGRAM_POINT_SIZE);

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

    pub fn tetrahedralize(&mut self) -> Vec<Tetrahedron> {
        let mut tetrahedrons = Vec::new();

        for faces in self.solids.chunks(self.faces_per_solid as usize) {
            // The index of the vertex that all tetrahedrons making up this solid
            // will connect to.
            let mut apex = u32::max_value();

            let previous_len = tetrahedrons.len();

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

                    // TODO
                    const PERMUTATIONS: [(usize, usize, usize); 2] = [(0, 1, 2), (2, 3, 0)];

                    for (a, b, c) in PERMUTATIONS.iter() {
                        // Next, form a tetrahedron with each triangle and the apex vertex.
                        tetrahedrons.push(Tetrahedron::new([
                            self.get_vertex(face_vertices[*a] as usize),
                            self.get_vertex(face_vertices[*b] as usize),
                            self.get_vertex(face_vertices[*c] as usize),
                            self.get_vertex(apex as usize),
                        ]));
                    }
                }
            }
            println!("Added {} tetrahedrons", tetrahedrons.len() - previous_len);
        }

        tetrahedrons
    }

    /// Pseudo-code:
    ///
    /// create `hyperplane`
    /// create new list of `points`
    /// create new list of `indices`
    ///
    /// for each `solid` in `polytope`
    ///     pick a `corner` that all tetrahedrons will terminate at
    ///     for each `face` in `solid`
    ///         if `face` does not contain `corner` then:
    ///             break `face` into two distinct triangles
    ///             for each triangle, connect it to `corner` to form a complete tetrahedron
    ///
    /// ...
    ///
    /// for each `tetrahedron`
    ///
    ///     set `intersections` to 0
    ///
    ///     for each `edge` in `tetrahedron`
    ///         if `edge` is cut by `hyperplane`
    ///             increment `intersections` and add point to `points`
    ///
    ///     if `intersections` is 3:
    ///         add 3 new entries to `indices` in any order
    ///     else if `intersections` is 4:
    ///         add 6 new entries to `indices` in ??? order // TODO
    ///     else
    ///         throw error
    ///
    /// Returns a slice with the proper vertices and edge indices.
    pub fn slice(&self, hyperplane: &Hyperplane) -> Option<Slice> {
        let rot = rotations::align();

        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();

        let debug = false;

        let mut last_intersection_count = 0;

        for (solid, faces) in self.solids
            .chunks(self.faces_per_solid as usize)
            .enumerate()
        {
            let mut intersections = Vec::new();
            let mut examined_edges = Vec::new();

            // Each solid has `faces_per_solid` indices, corresponding to entries
            // in this polytope's `faces` list. For example, the first solid in a
            // hypercube contains the following face indices: [0  1  2  3  4  5].
            for face in faces {
                // Each face has `edges_per_face` indices, corresponding to entries
                // in this polytope's `edges` list. For example, the first face in a
                // hypercube contains the following edge indices: [0  1  2  3].
                let idx_face_s = (*face * self.edges_per_face) as usize;
                let idx_face_e = (*face * self.edges_per_face + self.edges_per_face) as usize;
                let edges = &self.faces[idx_face_s..idx_face_e];

                for edge in edges {
                    if !examined_edges.contains(edge) {
                        // Grab the pair of vertex indices corresponding to this edge.
                        let idx_edge_s = (*edge * self.vertices_per_edge) as usize;
                        let idx_edge_e =
                            (*edge * self.vertices_per_edge + self.vertices_per_edge) as usize;
                        let pair = &self.edges[idx_edge_s..idx_edge_e];

                        // Grab the two vertices that form this edge.
                        let p0 = self.get_vertex(pair[0] as usize);
                        let p1 = self.get_vertex(pair[1] as usize);

                        //                        if (p0.w > d && p1.w < d) || (p0.w < d && p1.w > d) {
                        //
                        //                            let intersection = Vector4::new(
                        //                              p0.x + (p1.x - p0.x) * (d - p0.w) / (p1.w - p0.w),
                        //                              p0.y + (p1.y - p0.y) * (d - p0.w) / (p1.w - p0.w),
                        //                              p0.z + (p1.z - p0.z) * (d - p0.w) / (p1.w - p0.w),
                        //                              d
                        //                            );
                        //                            intersections.push(intersection);
                        //                        }
                        //
                        //                        if (p0.w - d).abs() + (p1.w - d).abs() <= 1e-6 {
                        //                            intersections.push(p0);
                        //                            intersections.push(p1);
                        //                        }

                        // Calculate whether or not there was an intersection between this
                        // edge and the 4-dimensional hyperplane.
                        let u =
                            -hyperplane.side(&p0) / (hyperplane.side(&p1) - hyperplane.side(&p0));
                        if u >= 0.0 && u <= 1.0 {
                            // Calculate the point of intersection in 4D.
                            let intersection = p0 + (p1 - p0) * u;

                            intersections.push(intersection);
                        }

                        examined_edges.push(*edge);
                    }
                }
            }

            let mut intersections_3d = Vec::new();
            for point in intersections.iter() {
                let point_transformed = rot * point;
                let point_3d = Vector3::new(
                    point_transformed.y,
                    point_transformed.z,
                    point_transformed.w,
                );
                intersections_3d.push(point_3d);
            }

            if intersections_3d.len() >= 3 {
                let mut centroid: Vector3<f32> = intersections_3d.iter().sum();
                centroid /= intersections_3d.len() as f32;

                let a = intersections_3d[0];
                let b = intersections_3d[1];
                let c = intersections_3d[2];

                // Calculate the normal of this polygon by taking the cross product
                // between two of its edges.
                let ab = b - a;
                let bc = c - b;
                let polygon_normal = bc.cross(ab).normalize();

                let mut first_edge = (a - centroid).normalize();

                let mut indices = Vec::new();

                for i in 1..intersections_3d.len() {
                    let p = intersections_3d[i];

                    let edge = (p - centroid).normalize();

                    let mut ang = first_edge.dot(edge);
                    ang = ang.max(-1.0).min(1.0);

                    let mut signed_angle = ang.acos();
                    if polygon_normal.dot(first_edge.cross(edge)) < 0.0 {
                        signed_angle *= -1.0;
                    }

                    indices.push((i, signed_angle));
                }
                indices.push((0, 0.0));
                indices.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

                for index in 0..indices.len() {
                    let i0 = indices[index].0;
                    let i1 = indices[(index + 1) % indices.len()].0;
                    all_indices.push((i0 + last_intersection_count) as u32);
                    all_indices.push((i1 + last_intersection_count) as u32);
                }

                for point in intersections.iter() {
                    all_vertices.extend_from_slice(&[point.x, point.y, point.z, point.w]);
                }

                last_intersection_count += intersections.len();
            }
        }

        if all_vertices.len() > 0 && all_indices.len() > 0 {
            return Some(Slice::new(all_vertices, all_indices));
        }
        None
    }
}
