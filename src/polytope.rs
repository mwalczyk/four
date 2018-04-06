use std::fs::File;
use std::io::{BufRead, BufReader};
use std::mem;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr;

use cgmath::{self, InnerSpace, Matrix4, Vector4, Zero};
use gl;
use gl::types::*;

pub enum Plane {
    XY,
    YZ,
    ZX,
    XW,
    YW,
    ZW,
}

/// Takes a 4D cross product between `u`, `v`, and `w`.
pub fn cross(u: &Vector4<f32>, v: &Vector4<f32>, w: &Vector4<f32>) -> Vector4<f32> {
    let a = (v[0] * w[1]) - (v[1] * w[0]);
    let b = (v[0] * w[2]) - (v[2] * w[0]);
    let c = (v[0] * w[3]) - (v[3] * w[0]);
    let d = (v[1] * w[2]) - (v[2] * w[1]);
    let e = (v[1] * w[3]) - (v[3] * w[1]);
    let f = (v[2] * w[3]) - (v[3] * w[2]);

    let result = Vector4::new(
        (u[1] * f) - (u[2] * e) + (u[3] * d),
        -(u[0] * f) + (u[2] * c) - (u[3] * b),
        (u[0] * e) - (u[1] * c) + (u[3] * a),
        -(u[0] * d) + (u[1] * b) - (u[2] * a),
    );
    result
}

/// The 4D equivalent of a quaternion is known as a rotor.
/// Reference: `https://math.stackexchange.com/questions/1402362/rotation-in-4d`
pub fn get_simple_rotation_matrix(plane: Plane, angle: f32) -> Matrix4<f32> {
    let c = angle.cos();
    let s = angle.sin();

    match plane {
        Plane::XY => Matrix4::from_cols(
            Vector4::new(c, -s, 0.0, 0.0),
            Vector4::new(s, c, 0.0, 0.0),
            Vector4::new(0.0, 0.0, 1.0, 0.0),
            Vector4::new(0.0, 0.0, 0.0, 1.0),
        ),
        Plane::YZ => Matrix4::from_cols(
            Vector4::new(1.0, 0.0, 0.0, 0.0),
            Vector4::new(0.0, c, -s, 0.0),
            Vector4::new(0.0, s, c, 0.0),
            Vector4::new(0.0, 0.0, 0.0, 1.0),
        ),
        Plane::ZX => Matrix4::from_cols(
            Vector4::new(c, 0.0, s, 0.0),
            Vector4::new(0.0, 1.0, 0.0, 0.0),
            Vector4::new(-s, 0.0, c, 0.0),
            Vector4::new(0.0, 0.0, 0.0, 1.0),
        ),
        Plane::XW => Matrix4::from_cols(
            Vector4::new(c, 0.0, 0.0, -s),
            Vector4::new(0.0, 1.0, 0.0, 0.0),
            Vector4::new(0.0, 0.0, 1.0, 0.0),
            Vector4::new(s, 0.0, 0.0, c),
        ),
        Plane::YW => Matrix4::from_cols(
            Vector4::new(1.0, 0.0, 0.0, 0.0),
            Vector4::new(0.0, c, 0.0, s),
            Vector4::new(0.0, 0.0, 1.0, 0.0),
            Vector4::new(0.0, -s, 0.0, c),
        ),
        Plane::ZW => Matrix4::from_cols(
            Vector4::new(1.0, 0.0, 0.0, 0.0),
            Vector4::new(0.0, 1.0, 0.0, 0.0),
            Vector4::new(0.0, 0.0, c, s),
            Vector4::new(0.0, 0.0, -s, c),
        ),
    }
}

/// Returns a "double rotation" matrix, which represents two planes of rotation.
/// The only fixed point is the origin. If `alpha` and `beta` are equal and non-zero,
/// then the rotation is called an isoclinic rotation.
///
/// Reference: `https://en.wikipedia.org/wiki/Plane_of_rotation#Double_rotations`
pub fn get_double_rotation_matrix(alpha: f32, beta: f32) -> Matrix4<f32> {
    let ca = alpha.cos();
    let sa = alpha.sin();
    let cb = beta.cos();
    let sb = beta.sin();

    Matrix4::from_cols(
        Vector4::new(ca, sa, 0.0, 0.0),
        Vector4::new(-sa, ca, 0.0, 0.0),
        Vector4::new(0.0, 0.0, cb, sb),
        Vector4::new(0.0, 0.0, -sb, cb),
    )
}

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

    pub fn get_vertex(&self, index: usize) -> Vector4<f32> {
        Vector4::new(
            self.vertices[index * 4 + 0],
            self.vertices[index * 4 + 1],
            self.vertices[index * 4 + 2],
            self.vertices[index * 4 + 3],
        )
    }

    pub fn get_vertices(&self) -> &Vec<f32> {
        &self.vertices
    }

    pub fn get_edges(&self) -> &Vec<u32> {
        &self.edges
    }

    pub fn draw(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);

            //gl::DrawArrays(gl::POINTS, 0, (self.vertices.len() / 4) as i32);

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

            // Create the vertex array object.
            gl::CreateVertexArrays(1, &mut self.vao);

            let mut size = (self.vertices.len() * mem::size_of::<f32>()) as GLsizeiptr;

            // Create the vertex buffer for holding position data.
            gl::CreateBuffers(1, &mut self.vbo);
            gl::NamedBufferData(
                self.vbo,
                size,
                self.vertices.as_ptr() as *const GLvoid,
                gl::DYNAMIC_DRAW,
            );

            // Create the index buffer.
            size = (self.edges.len() * mem::size_of::<u32>()) as GLsizeiptr;
            gl::CreateBuffers(1, &mut self.ebo);
            gl::NamedBufferData(
                self.ebo,
                size,
                self.edges.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            // Set up vertex attributes.
            let binding = 0;

            gl::EnableVertexArrayAttrib(self.vao, 0);
            gl::VertexArrayAttribFormat(self.vao, 0, 4, gl::FLOAT, gl::FALSE, 0);
            gl::VertexArrayAttribBinding(self.vao, 0, binding);

            gl::VertexArrayElementBuffer(self.vao, self.ebo);

            // Link vertex buffers to vertex attributes, via binding points.
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

    /// Pseudo-code:
    ///
    /// create `hyperplane`
    /// create new list of `points`
    ///
    /// for each `solid` in `polytope`
    ///     for each `face` in `solid`
    ///         for each `edge` in `face`
    ///             compute intersection between `edge` and `hyperplane`
    ///             if VALID add to `points`
    ///
    ///     compute proper ordering of `points` (based on signed angle)
    ///
    /// Returns a slice with the proper vertices and edge indices.
    pub fn slice(&self, mut n: Vector4<f32>, d: f32) -> Option<Slice> {
        n = n.normalize();

        let side = |p: Vector4<f32>| -> f32 { n.dot(p) + d };

        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();

        let debug = false;

        let mut last_intersection_count = 0;

        for (solid, faces) in self.solids
            .chunks(self.faces_per_solid as usize)
            .enumerate()
        {
            // Each solid has `faces_per_solid` indices, corresponding to entries
            // in this polytope's `faces` list. For example, the first solid in a
            // hypercube contains the following face indices: [0  1  2  3  4  5].
            let mut examined_edges = Vec::new();
            let mut intersections = Vec::new();
            for face in faces {
                if debug {
                    println!("  face: {}", face);
                }

                // Each face has `edges_per_face` indices, corresponding to entries
                // in this polytope's `edges` list. For example, the first face in a
                // hypercube contains the following edge indices: [0  1  2  3].
                let idx_face_s = (*face * self.edges_per_face) as usize;
                let idx_face_e = (*face * self.edges_per_face + self.edges_per_face) as usize;
                let edges = &self.faces[idx_face_s..idx_face_e];

                if debug {
                    println!("      edges for this face: {:?}", edges);
                }
                for edge in edges {
                    // The faces that make up this solid will have shared edges, so
                    // we want to make sure that we calculate an intersection *once*
                    // per unique edge.
                    if !examined_edges.contains(edge) {
                        // Grab the pair of vertex indices corresponding to this edge.
                        let idx_edge_s = (*edge * self.vertices_per_edge) as usize;
                        let idx_edge_e =
                            (*edge * self.vertices_per_edge + self.vertices_per_edge) as usize;
                        let pair = &self.edges[idx_edge_s..idx_edge_e];

                        if debug {
                            println!("      edge: {:?}", pair);
                        }
                        // Grab the two vertices that form this edge.
                        let p0 = self.get_vertex(pair[0] as usize);
                        let p1 = self.get_vertex(pair[1] as usize);

                        if debug {
                            println!("          vertex 0: {:?}", p0);
                            println!("          vertex 1: {:?}", p1);
                        }

                        // Calculate whether or not there was an intersection between this
                        // edge and the 4-dimensional hyperplane.
                        let u = -side(p0) / (side(p1) - side(p0));
                        if u >= 0.0 && u <= 1.0 {
                            // Calculate the point of intersection in 4D.
                            let intersection = p0 + (p1 - p0) * u;
                            intersections.push(intersection);
                        }

                        examined_edges.push(*edge);
                    }
                }
            }

            if intersections.len() >= 3 {
                let mut centroid: Vector4<f32> = intersections.iter().sum();
                centroid /= intersections.len() as f32;

                let a = intersections[0];
                let b = intersections[1];
                let c = intersections[2];
                let ab = b - a;
                let bc = c - b;
                let ca = a - c;
                let polygon_normal = cross(&ab, &bc, &ca);

                let mut first_edge = (a - centroid).normalize();

                let mut indices = Vec::new();
                for i in 1..intersections.len() {
                    let p = intersections[i];

                    let edge = (p - centroid).normalize();

                    let mut ang = first_edge.dot(edge);
                    ang = ang.max(-1.0).min(1.0);

                    let mut signed_angle = ang.acos();
                    if polygon_normal.dot(cross(&first_edge, &edge, &ab)) < 0.0 {
                        signed_angle *= -1.0;
                    }

                    indices.push((i, signed_angle));
                }
                indices.push((0, 0.0));
                indices.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());



                for item in indices.iter() {
                    let i = item.0;
                    let point = intersections[i];
                    all_vertices.extend_from_slice(&[point.x, point.y, point.z, point.w]);
                    all_indices.push((i + last_intersection_count) as u32);
                }

                last_intersection_count = intersections.len();
                //println!("{:?}", indices);
            }

            //            println!(
            //                "{} intersections found for solid {}",
            //                intersections.len(), solid
            //            );
        }
        //println!("-------------------------------");

        if all_vertices.len() > 0 && all_indices.len() > 0 {
            return Some(Slice::new(all_vertices, all_indices));
        }
        None
    }
}

pub struct Slice {
    vertices: Vec<f32>,
    edges: Vec<u32>,
    vao: u32,
    vbo: u32,
    ebo: u32,
}

impl Slice {
    pub fn new(vertices: Vec<f32>, edges: Vec<u32>) -> Slice {
        let mut slice = Slice {
            vertices,
            edges,
            vao: 0,
            vbo: 0,
            ebo: 0,
        };

        slice.init_render_objects();
        slice
    }

    fn init_render_objects(&mut self) {
        unsafe {
            // Create the vertex array object.
            gl::CreateVertexArrays(1, &mut self.vao);

            let mut size = (self.vertices.len() * mem::size_of::<f32>()) as GLsizeiptr;

            // Create the vertex buffer for holding position data.
            gl::CreateBuffers(1, &mut self.vbo);
            gl::NamedBufferData(
                self.vbo,
                size,
                self.vertices.as_ptr() as *const GLvoid,
                gl::DYNAMIC_DRAW,
            );

            // Create the index buffer.
            size = (self.edges.len() * mem::size_of::<u32>()) as GLsizeiptr;
            gl::CreateBuffers(1, &mut self.ebo);
            gl::NamedBufferData(
                self.ebo,
                size,
                self.edges.as_ptr() as *const GLvoid,
                gl::DYNAMIC_DRAW,
            );

            // Set up vertex attributes.
            let binding = 0;

            gl::EnableVertexArrayAttrib(self.vao, 0);
            gl::VertexArrayAttribFormat(self.vao, 0, 4, gl::FLOAT, gl::FALSE, 0);
            gl::VertexArrayAttribBinding(self.vao, 0, binding);

            gl::VertexArrayElementBuffer(self.vao, self.ebo);

            // Link vertex buffers to vertex attributes, via binding points.
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

    pub fn draw(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            if self.vertices.len() > 0 {
                gl::DrawArrays(gl::POINTS, 0, (self.vertices.len() / 4) as i32);
            }

            if self.edges.len() > 0 {
                gl::DrawElements(
                    gl::LINES,
                    self.edges.len() as i32,
                    gl::UNSIGNED_INT,
                    ptr::null(),
                );
            }
        }
    }
}
