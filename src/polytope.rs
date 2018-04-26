use std::fs::File;
use std::io::{BufRead, BufReader};
use std::mem;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr;

use cgmath::{self, Array, ElementWise, InnerSpace, Vector3, Vector4, Zero};
use gl;
use gl::types::*;

use hyperplane::Hyperplane;
use rotations;
use tetrahedron::Tetrahedron;

pub enum Definition {
    Cell8,
    Cell24,
    Cell120,
    Cell600,
    Hypersphere,
}

impl Definition {}

pub struct Polytope {
    vertices: Vec<Vector4<f32>>,
    edges: Vec<u32>,
    faces: Vec<u32>,
    solids: Vec<u32>,
    components_per_vertex: u32,
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

//        // Load solid data (6 entries per solid).
//        reader.read_line(&mut entry_count);
//        number_of_entries = entry_count.trim().parse().unwrap();
//        let mut solids = Vec::with_capacity(number_of_entries * 6);
//
//        for _ in 0..number_of_entries {
//            let mut line = String::new();
//            reader.read_line(&mut line);
//
//            for entry in line.split_whitespace() {
//                let data: u32 = entry.trim().parse().unwrap();
//                solids.push(data);
//            }
//        }
//
//        println!(
//            "Loaded file with {} vertices, {} edges, {} faces, and {} solids",
//            vertices.len(),
//            edges.len() / 2,
//            faces.len() / 4,
//            solids.len() / 6
//        );

        let mut polytope = Polytope {
            vertices,
            edges,
            faces,
            solids: Vec::new(),
            components_per_vertex: 4,
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

    /// Returns the number of unique vertices in this mesh.
    pub fn get_number_of_vertices(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the number of unique edges in this mesh.
    pub fn get_number_of_edges(&self) -> usize {
        self.edges.len() / self.vertices_per_edge as usize
    }

    /// Returns the number of unique faces in this mesh.
    pub fn get_number_of_faces(&self) -> usize {
        self.faces.len() / self.edges_per_face as usize
    }

    /// Returns the `i`th vertex of this polytope.
    pub fn get_vertex(&self, i: u32) -> Vector4<f32> {
        self.vertices[i as usize]
    }

    /// Returns an unordered tuple of the two vertices that make up the `i`th
    /// edge of this polytope.
    pub fn get_vertices_for_edge(&self, i: u32) -> (Vector4<f32>, Vector4<f32>) {
        let idx_edge_s = (i * self.vertices_per_edge) as usize;
        let idx_edge_e = (i * self.vertices_per_edge + self.vertices_per_edge) as usize;
        let pair = &self.edges[idx_edge_s..idx_edge_e];

        (self.get_vertex(pair[0]), self.get_vertex(pair[1]))
    }

    /// Returns an unordered list of the unique vertices that make up the `i`th
    /// face of this polytope.
    pub fn get_vertices_for_face(&self, i: u32) -> Vec<Vector4<f32>> {
        //let mut visited_vertices = Vec::new();
        let mut vertices = Vec::new();

        let idx_face_s = (i * self.edges_per_face) as usize;
        let idx_face_e = (i * self.edges_per_face + self.edges_per_face) as usize;
        let edges = &self.faces[idx_face_s..idx_face_e];

        for edge in edges {
            let (a, b) = self.get_vertices_for_edge(*edge);

            // We want to make sure that we don't add the same vertex to the
            // list multiple times.
            if !vertices.contains(&a) {
                vertices.push(a);
            }
            if !vertices.contains(&b) {
                vertices.push(b);
            }
        }

        vertices
    }

    pub fn get_vertices_for_solids(&self) {
        // TODO
    }

    /// The H-representation of a convex polytope is the list of hyperplanes whose
    /// intersection produces the desired shape.
    ///
    /// See: `https://en.wikipedia.org/wiki/Convex_polytope#Intersection_of_half-spaces`
    pub fn get_h_representation(&self) -> Vec<Hyperplane> {

        // ~ 1.73205
        let r = 3.0.sqrt();

        // sqrt(3) = 1.73205
        // sqrt(5) = 2.23606
        // phi = 1.61803

        // 2 - 1.41421 = 0.58579
        // self.normal.dot(*point) + 1.73205



        // THIS is our radius = 2 * sqrt(2) / 2
        // described here: http://mathworld.wolfram.com/120-Cell.html
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

        println!("{:?}", solids);
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

    /// Performs of a tetrahedral decomposition of the polytope.
    pub fn tetrahedralize(&mut self, hyperplane: &Hyperplane) -> Vec<Tetrahedron> {
        use std::f32;

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

        for (solid, faces) in self.gather_solids().iter().enumerate() {
            // The vertex that all tetrahedrons making up this solid will connect to.
            let mut apex = Vector4::from_value(f32::MAX);

            // Iterate over each face of the current cell.
            for face in faces {
                // Retrieve the indices of all of the edges that make up this face.
                let idx_face_s = (*face * self.edges_per_face) as usize;
                let idx_face_e = (*face * self.edges_per_face + self.edges_per_face) as usize;
                let edges = &self.faces[idx_face_s..idx_face_e];

                let face_vertices = self.get_vertices_for_face(*face);

                if apex.x == f32::MAX {
                    apex = face_vertices[0];
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
                    let v0 = face_vertices[0];
                    let v1 = face_vertices[1];
                    let v2 = face_vertices[2];
                    let edge_1_0 = v1 - v0;
                    let edge_2_0 = v2 - v0;
                    let edge_2_1 = v2 - v1;

                    let face_hyperplane =
                        Hyperplane::new(rotations::cross(&edge_1_0, &edge_2_0, &edge_2_1), 0.0);

                    // Collect all 4D vertices and sort.
                    let face_vertices_sorted =
                        rotations::sort_points_on_plane(&face_vertices, &face_hyperplane);

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
                            get_color_for_tetrahedron(solid as f32 / 8.0),
                        ));
                    }
                }
            }
        }

        tetrahedrons
    }
}
