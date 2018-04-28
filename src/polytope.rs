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
use rotations::{self, Plane};
use tetrahedron::Tetrahedron;

struct Definition {
    components_per_vertex: u32,
    vertices_per_edge: u32,
    vertices_per_face: u32,
    vertices_per_solid: u32
}

pub enum Polychoron {
    Cell8,
    Cell24,
    Cell120,
    Cell600,
}

impl Polychoron {
    pub fn get_definition(&self) -> Definition {
        Definition {
            components_per_vertex: 0,
            vertices_per_edge: 0,
            vertices_per_face: 0,
            vertices_per_solid: 0
        }
    }
}

pub struct Polytope {
    pub vertices: Vec<Vector4<f32>>,
    edges: Vec<u32>,
    faces: Vec<u32>,
    solids: Vec<u32>,
    components_per_vertex: u32,
    vertices_per_edge: u32,
    vertices_per_face: u32,
    vertices_per_solid: u32,
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
    /// ...
    ///
    /// number_of_edges
    /// v0 v1
    /// v2 v3
    /// ...
    ///
    /// number_of_faces
    /// v0 v1 v2 v3
    /// v4 v5 v6 v7
    /// ...
    ///
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
            let vertex = Vector4::new(x, y, z, w); // TODO: wtf how does this cause so much: - Vector4::from_value(0.001);

            vertices.push(vertex);
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


        let mut polytope = Polytope {
            vertices,
            edges,
            faces,
            solids: Vec::new(),
            components_per_vertex: 4,
            vertices_per_edge: 2,
//
//            vertices_per_face: 4,
//            vertices_per_solid: 6,
            vertices_per_face: 5,
            vertices_per_solid: 20,
            vao: 0,
            vbo: 0,
            ebo: 0,
        };

        println!(
            "Loaded file with {} vertices, {} edges, {} faces",
            polytope.vertices.len(),
            polytope.edges.len() / polytope.vertices_per_edge as usize,
            polytope.faces.len() / polytope.vertices_per_face as usize,
        );

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
        self.faces.len() / self.vertices_per_face as usize
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
        let idx_face_s = (i * self.vertices_per_face) as usize;
        let idx_face_e = (i * self.vertices_per_face + self.vertices_per_face) as usize;
        let vertex_ids = &self.faces[idx_face_s..idx_face_e];

        let vertices = vertex_ids
            .iter()
            .map(|id| self.get_vertex(*id))
            .collect::<Vec<_>>();

        vertices
    }

    /// Returns an unordered list of the unique vertices that make up the `i`th
    /// solid of this polytope.
    pub fn get_vertices_for_solid(&self, i: u32) -> Vec<Vector4<f32>> {
        let idx_solid_s = (i * self.vertices_per_solid) as usize;
        let idx_solid_e = (i * self.vertices_per_solid + self.vertices_per_solid) as usize;
        let vertex_ids = &self.solids[idx_solid_s..idx_solid_e];

        vertex_ids
            .iter()
            .map(|id| self.get_vertex(*id))
            .collect::<Vec<_>>()
    }

    /// The H-representation of a convex polytope is the list of hyperplanes whose
    /// intersection produces the desired shape.
    ///
    /// See: `https://en.wikipedia.org/wiki/Convex_polytope#Intersection_of_half-spaces`
    pub fn get_h_representation(&self) -> Vec<Hyperplane> {

        // The circumradius of the 120-cell is: 2√2, ~2.828

        // The inner "radius" of this particular 120-cell is: -2 * φ^2
        let golden_ratio: f32 = (1.0 + 5.0f32.sqrt()) / 2.0;
        let displacement = -golden_ratio.powf(2.0) * 2.0;

        let mut representation = vec![
            Hyperplane::new(Vector4::new(2.0, 0.0, 0.0, 0.0), displacement),
            Hyperplane::new(Vector4::new(-2.0, 0.0, 0.0, 0.0), displacement),
            Hyperplane::new(Vector4::new(0.0, 2.0, 0.0, 0.0), displacement),
            Hyperplane::new(Vector4::new(0.0, -2.0, 0.0, 0.0), displacement),
            Hyperplane::new(Vector4::new(0.0, 0.0, 2.0, 0.0), displacement),
            Hyperplane::new(Vector4::new(0.0, 0.0, -2.0, 0.0), displacement),
            Hyperplane::new(Vector4::new(0.0, 0.0, 0.0, 2.0), displacement),
            Hyperplane::new(Vector4::new(0.0, 0.0, 0.0, -2.0), displacement),

            Hyperplane::new(Vector4::new(2.618033988749894, 1.0, 0.0, -1.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(2.618033988749894, 0.0, 1.618033988749894, -1.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(2.618033988749894, 0.0, -1.618033988749894, -1.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(2.618033988749894, -1.0, 0.0, -1.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(2.618033988749894, -1.618033988749894, -1.0, 0.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(1.618033988749894, 2.618033988749894, 0.0, -1.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(1.618033988749894, 1.0, -2.618033988749894, 0.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(1.618033988749894, 0.0, 1.0, -2.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(1.618033988749894, 0.0, -1.0, -2.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(1.618033988749894, -1.0, -2.618033988749894, 0.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(1.618033988749894, -2.618033988749894, 0.0, -1.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(1.0, 2.618033988749894, -1.618033988749894, 0.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(1.0, 1.618033988749894, 0.0, -2.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(1.0, 0.0, 2.618033988749894, -1.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(1.0, 0.0, -2.618033988749894, -1.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(1.0, -1.618033988749894, 0.0, -2.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(0.0, 2.618033988749894, 1.0, -1.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(0.0, 2.618033988749894, -1.0, -1.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(0.0, 1.618033988749894, 2.618033988749894, -1.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(0.0, 1.618033988749894, -2.618033988749894, -1.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(0.0, 1.0, 1.618033988749894, -2.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(0.0, 1.0, -1.618033988749894, -2.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(0.0, -1.0, 1.618033988749894, -2.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(0.0, -1.0, -1.618033988749894, -2.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(0.0, -1.618033988749894, 2.618033988749894, -1.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(0.0, -1.618033988749894, -2.618033988749894, -1.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(0.0, -2.618033988749894, 1.0, -1.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(0.0, -2.618033988749894, -1.0, -1.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(-1.0, 1.618033988749894, 0.0, 2.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(-1.0, 0.0, 2.618033988749894, -1.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(-1.0, 0.0, -2.618033988749894, -1.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(-1.0, -1.618033988749894, 0.0, -2.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(-1.0, -2.618033988749894, -1.618033988749894, 0.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(-1.618033988749894, 2.618033988749894, 0.0, -1.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(-1.618033988749894, 1.0, -2.618033988749894, 0.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(-1.618033988749894, 0.0, 1.0, -2.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(-1.618033988749894, 0.0, -1.0, -2.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(-1.618033988749894, -1.0, -2.618033988749894, 0.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(-1.618033988749894, -2.618033988749894, 0.0, -1.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(-2.618033988749894, 1.618033988749894, -1.0, 0.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(-2.618033988749894, 1.0, 0.0, -1.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(-2.618033988749894, 0.0, 1.618033988749894, -1.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(-2.618033988749894, 0.0, -1.618033988749894, -1.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(-2.618033988749894, -1.0, -0.0, -1.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(-2.618033988749894, -1.618033988749894, -1.0, -0.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(-2.618033988749894, -1.618033988749894, 1.0, -0.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(-2.618033988749894, -1.0, 0.0, 1.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(-2.618033988749894, 0.0, -1.618033988749894, 1.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(-2.618033988749894, 0.0, 1.618033988749894, 1.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(-2.618033988749894, 1.0, 0.0, 1.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(-2.618033988749894, 1.618033988749894, 1.0, 0.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(-1.618033988749894, -2.618033988749894, 0.0, 1.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(-1.618033988749894, -1.0, 2.618033988749894, 0.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(-1.618033988749894, -0.0, -1.0, 2.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(-1.618033988749894, 0.0, 1.0, 2.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(-1.618033988749894, 1.0, 2.618033988749894, 0.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(-1.618033988749894, 2.618033988749894, 0.0, 1.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(-1.0, -2.618033988749894, 1.618033988749894, 0.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(-1.0, -1.618033988749894, 0.0, 2.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(-1.0, 0.0, -2.618033988749894, 1.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(-1.0, 0.0, 2.618033988749894, 1.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(-1.0, 1.618033988749894, 0.0, -2.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(-1.0, 2.618033988749894, -1.618033988749894, 0.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(-1.0, 2.618033988749894, 1.618033988749894, 0.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(0.0, -2.618033988749894, -1.0, 1.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(0.0, -2.618033988749894, 1.0, 1.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(0.0, -1.618033988749894, -2.618033988749894, 1.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(0.0, -1.618033988749894, 2.618033988749894, 1.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(0.0, -1.0, -1.618033988749894, 2.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(0.0, -1.0, 1.618033988749894, 2.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(0.0, 1.0, -1.618033988749894, 2.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(0.0, 1.0, 1.618033988749894, 2.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(0.0, 1.618033988749894, -2.618033988749894, 1.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(0.0, 1.618033988749894, 2.618033988749894, 1.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(0.0, 2.618033988749894, -1.0, 1.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(0.0, 2.618033988749894, 1.0, 1.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(1.0, -2.618033988749894, -1.618033988749894, 0.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(1.0, -2.618033988749894, 1.618033988749894, 0.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(1.0, -1.618033988749894, 0.0, 2.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(1.0, 0.0, -2.618033988749894, 1.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(1.0, 0.0, 2.618033988749894, 1.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(1.0, 1.618033988749894, 0.0, 2.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(1.0, 2.618033988749894, 1.618033988749894, 0.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(1.618033988749894, -2.618033988749894, 0.0, 1.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(1.618033988749894, -1.0, 2.618033988749894, 0.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(1.618033988749894, 0.0, -1.0, 2.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(1.618033988749894, 0.0, 1.0, 2.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(1.618033988749894, 1.0, 2.618033988749894, 0.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(1.618033988749894, 2.618033988749894, 0.0, 1.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(2.618033988749894, -1.618033988749894, 1.0, 0.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(2.618033988749894, -1.0, 0.0, 1.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(2.618033988749894, 0.0, -1.618033988749894, 1.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(2.618033988749894, 0.0, 1.618033988749894, 1.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(2.618033988749894, 1.0, 0.0, 1.618033988749894), -8.472135954999578),
            Hyperplane::new(Vector4::new(2.618033988749894, 1.618033988749894, -1.0, 0.0), -8.472135954999578),
            Hyperplane::new(Vector4::new(2.618033988749894, 1.618033988749894, 1.0, 0.0), -8.472135954999578),

//            Hyperplane::new(Vector4::new(0.0, 1.0, 0.618034, 1.61803), displacement),
//            Hyperplane::new(Vector4::new(0.0, -1.0, 0.618034, 1.61803), displacement),
//            Hyperplane::new(Vector4::new(0.0, 1.0, -0.618034, 1.61803), displacement),
//            Hyperplane::new(Vector4::new(0.0, 1.0, 0.618034, -1.61803), displacement),
//            Hyperplane::new(Vector4::new(0.0, -1.0, -0.618034, 1.61803), displacement),
//            Hyperplane::new(Vector4::new(0.0, -1.0, 0.618034, -1.61803), displacement),
//            Hyperplane::new(Vector4::new(0.0, 1.0, -0.618034, -1.61803), displacement),
//            Hyperplane::new(Vector4::new(0.0, -1.0, -0.618034, -1.61803), displacement),
//            Hyperplane::new(Vector4::new(0.0, 0.618034, 1.61803, 1.0), displacement),
//            Hyperplane::new(Vector4::new(0.0, -0.618034, 1.61803, 1.0), displacement),
//            Hyperplane::new(Vector4::new(0.0, 0.618034, -1.61803, 1.0), displacement),
//            Hyperplane::new(Vector4::new(0.0, 0.618034, 1.61803, -1.0), displacement),
//            Hyperplane::new(Vector4::new(0.0, -0.618034, -1.61803, 1.0), displacement),
//            Hyperplane::new(Vector4::new(0.0, -0.618034, 1.61803, -1.0), displacement),
//            Hyperplane::new(Vector4::new(0.0, 0.618034, -1.61803, -1.0), displacement),
//            Hyperplane::new(Vector4::new(0.0, -0.618034, -1.61803, -1.0), displacement),
//            Hyperplane::new(Vector4::new(0.0, 1.61803, 1.0, 0.618034), displacement),
//            Hyperplane::new(Vector4::new(0.0, -1.61803, 1.0, 0.618034), displacement),
//            Hyperplane::new(Vector4::new(0.0, 1.61803, -1.0, 0.618034), displacement),
//            Hyperplane::new(Vector4::new(0.0, 1.61803, 1.0, -0.618034), displacement),
//            Hyperplane::new(Vector4::new(0.0, -1.61803, -1.0, 0.618034), displacement),
//            Hyperplane::new(Vector4::new(0.0, -1.61803, 1.0, -0.618034), displacement),
//            Hyperplane::new(Vector4::new(0.0, 1.61803, -1.0, -0.618034), displacement),
//            Hyperplane::new(Vector4::new(0.0, -1.61803, -1.0, -0.618034), displacement),
//            Hyperplane::new(Vector4::new(1.0, 0.0, 1.61803, 0.618034), displacement),
//            Hyperplane::new(Vector4::new(-1.0, 0.0, 1.61803, 0.618034), displacement),
//            Hyperplane::new(Vector4::new(1.0, 0.0, -1.61803, 0.618034), displacement),
//            Hyperplane::new(Vector4::new(1.0, 0.0, 1.61803, -0.618034), displacement),
//            Hyperplane::new(Vector4::new(-1.0, 0.0, -1.61803, 0.618034), displacement),
//            Hyperplane::new(Vector4::new(-1.0, 0.0, 1.61803, -0.618034), displacement),
//            Hyperplane::new(Vector4::new(1.0, 0.0, -1.61803, -0.618034), displacement),
//            Hyperplane::new(Vector4::new(-1.0, 0.0, -1.61803, -0.618034), displacement),
//            Hyperplane::new(Vector4::new(0.618034, 0.0, 1.0, 1.61803), displacement),
//            Hyperplane::new(Vector4::new(-0.618034, 0.0, 1.0, 1.61803), displacement),
//            Hyperplane::new(Vector4::new(0.618034, 0.0, -1.0, 1.61803), displacement),
//            Hyperplane::new(Vector4::new(0.618034, 0.0, 1.0, -1.61803), displacement),
//            Hyperplane::new(Vector4::new(-0.618034, 0.0, -1.0, 1.61803), displacement),
//            Hyperplane::new(Vector4::new(-0.618034, 0.0, 1.0, -1.61803), displacement),
//            Hyperplane::new(Vector4::new(0.618034, 0.0, -1.0, -1.61803), displacement),
//            Hyperplane::new(Vector4::new(-0.618034, 0.0, -1.0, -1.61803), displacement),
//            Hyperplane::new(Vector4::new(1.61803, 0.0, 0.618034, 1.0), displacement),
//            Hyperplane::new(Vector4::new(-1.61803, 0.0, 0.618034, 1.0), displacement),
//            Hyperplane::new(Vector4::new(1.61803, 0.0, -0.618034, 1.0), displacement),
//            Hyperplane::new(Vector4::new(1.61803, 0.0, 0.618034, -1.0), displacement),
//            Hyperplane::new(Vector4::new(-1.61803, 0.0, -0.618034, 1.0), displacement),
//            Hyperplane::new(Vector4::new(-1.61803, 0.0, 0.618034, -1.0), displacement),
//            Hyperplane::new(Vector4::new(1.61803, 0.0, -0.618034, -1.0), displacement),
//            Hyperplane::new(Vector4::new(-1.61803, 0.0, -0.618034, -1.0), displacement),
//            Hyperplane::new(Vector4::new(0.618034, 1.61803, 0.0, 1.0), displacement),
//            Hyperplane::new(Vector4::new(-0.618034, 1.61803, 0.0, 1.0), displacement),
//            Hyperplane::new(Vector4::new(0.618034, -1.61803, 0.0, 1.0), displacement),
//            Hyperplane::new(Vector4::new(0.618034, 1.61803, 0.0, -1.0), displacement),
//            Hyperplane::new(Vector4::new(-0.618034, -1.61803, 0.0, 1.0), displacement),
//            Hyperplane::new(Vector4::new(-0.618034, 1.61803, 0.0, -1.0), displacement),
//            Hyperplane::new(Vector4::new(0.618034, -1.61803, 0.0, -1.0), displacement),
//            Hyperplane::new(Vector4::new(-0.618034, -1.61803, 0.0, -1.0), displacement),
//            Hyperplane::new(Vector4::new(1.61803, 1.0, 0.0, 0.618034), displacement),
//            Hyperplane::new(Vector4::new(-1.61803, 1.0, 0.0, 0.618034), displacement),
//            Hyperplane::new(Vector4::new(1.61803, -1.0, 0.0, 0.618034), displacement),
//            Hyperplane::new(Vector4::new(1.61803, 1.0, 0.0, -0.618034), displacement),
//            Hyperplane::new(Vector4::new(-1.61803, -1.0, 0.0, 0.618034), displacement),
//            Hyperplane::new(Vector4::new(-1.61803, 1.0, 0.0, -0.618034), displacement),
//            Hyperplane::new(Vector4::new(1.61803, -1.0, 0.0, -0.618034), displacement),
//            Hyperplane::new(Vector4::new(-1.61803, -1.0, 0.0, -0.618034), displacement),
//            Hyperplane::new(Vector4::new(1.0, 0.618034, 0.0, 1.61803), displacement),
//            Hyperplane::new(Vector4::new(-1.0, 0.618034, 0.0, 1.61803), displacement),
//            Hyperplane::new(Vector4::new(1.0, -0.618034, 0.0, 1.61803), displacement),
//            Hyperplane::new(Vector4::new(1.0, 0.618034, 0.0, -1.61803), displacement),
//            Hyperplane::new(Vector4::new(-1.0, -0.618034, 0.0, 1.61803), displacement),
//            Hyperplane::new(Vector4::new(-1.0, 0.618034, 0.0, -1.61803), displacement),
//            Hyperplane::new(Vector4::new(1.0, -0.618034, 0.0, -1.61803), displacement),
//            Hyperplane::new(Vector4::new(-1.0, -0.618034, 0.0, -1.61803), displacement),
//            Hyperplane::new(Vector4::new(1.61803, 0.618034, 1.0, 0.0), displacement),
//            Hyperplane::new(Vector4::new(-1.61803, 0.618034, 1.0, 0.0), displacement),
//            Hyperplane::new(Vector4::new(1.61803, -0.618034, 1.0, 0.0), displacement),
//            Hyperplane::new(Vector4::new(1.61803, 0.618034, -1.0, 0.0), displacement),
//            Hyperplane::new(Vector4::new(-1.61803, -0.618034, 1.0, 0.0), displacement),
//            Hyperplane::new(Vector4::new(-1.61803, 0.618034, -1.0, 0.0), displacement),
//            Hyperplane::new(Vector4::new(1.61803, -0.618034, -1.0, 0.0), displacement),
//            Hyperplane::new(Vector4::new(-1.61803, -0.618034, -1.0, 0.0), displacement),
//            Hyperplane::new(Vector4::new(1.0, 1.61803, 0.618034, 0.0), displacement),
//            Hyperplane::new(Vector4::new(-1.0, 1.61803, 0.618034, 0.0), displacement),
//            Hyperplane::new(Vector4::new(1.0, -1.61803, 0.618034, 0.0), displacement),
//            Hyperplane::new(Vector4::new(1.0, 1.61803, -0.618034, 0.0), displacement),
//            Hyperplane::new(Vector4::new(-1.0, -1.61803, 0.618034, 0.0), displacement),
//            Hyperplane::new(Vector4::new(-1.0, 1.61803, -0.618034, 0.0), displacement),
//            Hyperplane::new(Vector4::new(1.0, -1.61803, -0.618034, 0.0), displacement),
//            Hyperplane::new(Vector4::new(-1.0, -1.61803, -0.618034, 0.0), displacement),
//            Hyperplane::new(Vector4::new(0.618034, 1.0, 1.61803, 0.0), displacement),
//            Hyperplane::new(Vector4::new(-0.618034, 1.0, 1.61803, 0.0), displacement),
//            Hyperplane::new(Vector4::new(0.618034, -1.0, 1.61803, 0.0), displacement),
//            Hyperplane::new(Vector4::new(0.618034, 1.0, -1.61803, 0.0), displacement),
//            Hyperplane::new(Vector4::new(-0.618034, -1.0, 1.61803, 0.0), displacement),
//            Hyperplane::new(Vector4::new(-0.618034, 1.0, -1.61803, 0.0), displacement),
//            Hyperplane::new(Vector4::new(0.618034, -1.0, -1.61803, 0.0), displacement),
//            Hyperplane::new(Vector4::new(-0.618034, -1.0, -1.61803, 0.0), displacement),


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
    pub fn gather_solids(&self) -> Vec<(Hyperplane, Vec<u32>)> {
        let mut solids = Vec::new();

        for hyperplane in self.get_h_representation().iter() {
            let mut faces_in_hyperplane = Vec::new();

            // Iterate over all of the faces of this polytope. For the 120-cell, for example,
            // there are 720 faces, each of which has 5 vertices associated with it.
            assert_eq!(self.get_number_of_faces(), 720);

            for face_index in 0..self.get_number_of_faces() {

                let face_vertices = self.get_vertices_for_face(face_index as u32);

                assert_eq!(face_vertices.len(), 5);

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

        for (solid, plane_and_faces) in self.gather_solids().iter().enumerate() {

            // The vertex that all tetrahedrons making up this solid will connect to.
            let mut apex = Vector4::from_value(f32::MAX);

            let (hyperplane, faces) = plane_and_faces;
            let mut prev_len = tetrahedrons.len();

            // Iterate over each face of the current cell.
            for face in faces {

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
                    // Collect all 4D vertices and sort.
                    let face_vertices_sorted =
                        rotations::sort_points_on_plane(&face_vertices, &hyperplane);

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
                            get_color_for_tetrahedron(solid as f32 / 600.0),
                        ));
                    }
                }
            }

            println!("{} tetrahedrons found for solid {}", tetrahedrons.len()-prev_len, solid);
        }

        tetrahedrons
    }
}
