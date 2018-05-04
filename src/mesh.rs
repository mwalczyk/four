use std::f32;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::mem;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr;

use cgmath::{self, Matrix4, Vector3, Vector4, Array, InnerSpace, Zero};
use gl;
use gl::types::*;

use hyperplane::Hyperplane;
use polychora::{Definition, Polychoron};
use program::Program;
use rotations::{self, Plane};
use tetrahedron::Tetrahedron;
use utilities;

/// A 4-dimensional mesh.
pub struct Mesh {
    pub vertices: Vec<Vector4<f32>>,
    pub edges: Vec<u32>,
    pub faces: Vec<u32>,
    pub polychoron: Polychoron,
    pub def: Definition,
    pub tetrahedrons: Vec<Tetrahedron>,
    pub slice_program: Program,
    vao: u32,
    vbo: u32,
    ebo: u32,
    ssbo_tetrahedra: u32,
    vbo_slice_colors: u32,
    ssbo_slice_vertices: u32,
    ssbo_slice_indices: u32,
}

impl Mesh {
    pub fn new(polychoron: Polychoron) -> Mesh {
        let cs = utilities::load_file_as_string(Path::new("shaders/compute_slice.glsl"));

        let mut mesh = Mesh {
            vertices: polychoron.get_vertices(),
            edges: polychoron.get_edges(),
            faces: polychoron.get_faces(),
            polychoron,
            def: polychoron.get_definition(),
            tetrahedrons: Vec::new(),
            slice_program: Program::single_stage(cs).unwrap(),
            vao: 0,
            vbo: 0,
            ebo: 0,
            ssbo_tetrahedra: 0,
            vbo_slice_colors: 0,
            ssbo_slice_vertices: 0,
            ssbo_slice_indices: 0,
        };

        mesh.tetrahedralize();
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

    /// Given the H-representation of this polytope, return a list of lists, where
    /// each sub-list contains the indices of all faces that are inside of the `i`th
    /// hyperplane.
    pub fn gather_cells(&self) -> Vec<(Hyperplane, Vec<u32>)> {
        let mut solids = Vec::new();

        for hyperplane in self.polychoron.get_h_representation().iter() {
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

            gl::VertexArrayElementBuffer(self.vao, self.ssbo_slice_indices);

            let binding = 0;
            let offset = 0;
            gl::VertexArrayVertexBuffer(
                self.vao,
                binding,
                self.ssbo_slice_vertices,
                offset,
                (mem::size_of::<Vector4<f32>>() as usize) as i32,
            );

            gl::VertexArrayVertexBuffer(self.vao, 1, self.vbo_slice_colors, 0, (mem::size_of::<Vector4<f32>>() as usize) as i32);

            gl::DrawElements(
                gl::TRIANGLE_STRIP,
                (6 * 3240) as i32,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
        }
    }

    fn init_render_objects(&mut self) {
        unsafe {
            gl::CreateVertexArrays(1, &mut self.vao);

            let binding = 0;
            gl::EnableVertexArrayAttrib(self.vao, 0);
            gl::VertexArrayAttribFormat(self.vao, 0, self.def.components_per_vertex as i32, gl::FLOAT, gl::FALSE, 0); // positions
            gl::VertexArrayAttribBinding(self.vao, 0, binding);

            gl::EnableVertexArrayAttrib(self.vao, 1);
            gl::VertexArrayAttribFormat(self.vao, 1, self.def.components_per_vertex as i32, gl::FLOAT, gl::FALSE, 0); // colors
            gl::VertexArrayAttribBinding(self.vao, 1, binding + 1);

            gl::VertexArrayVertexBuffer(self.vao, 1, self.vbo_slice_colors, 0, (mem::size_of::<Vector4<f32>>() as usize) as i32);

            gl::Enable(gl::PRIMITIVE_RESTART);
            gl::PrimitiveRestartIndex(0xFFFF);
            println!("Enabled primitive restart with index: {}", 0xFFFF);

            // Initialize the SSBO that will hold this mesh's tetrahedra.
            let mut vertices = Vec::new();
            let mut colors = Vec::new();
            for tetra in self.tetrahedrons.iter() {
                vertices.extend_from_slice(&tetra.vertices);

                // TODO: for now, we have to do this 4 times. Probably something to do with vertex attributes.
                colors.push(tetra.cell_centroid);
                colors.push(tetra.cell_centroid);
                colors.push(tetra.cell_centroid);
                colors.push(tetra.cell_centroid);
            }
            let total_tetrahedra = self.def.cells * (self.def.faces_per_cell - 3) * (self.def.vertices_per_face - 2);
            const VERTICES_PER_TETRAHEDRON: usize = 4;

            let vertices_size = mem::size_of::<Vector4<f32>>() * 4 * total_tetrahedra as usize;
            let colors_size = mem::size_of::<Vector4<f32>>() * 4 * total_tetrahedra as usize;
            println!("Size of data store for {} : {}", total_tetrahedra, vertices.len());

            // The SSBO that will be bound at index 0 and read from.
            gl::CreateBuffers(1, &mut self.ssbo_tetrahedra);
            gl::NamedBufferData(
                self.ssbo_tetrahedra,
                vertices_size as isize,
                vertices.as_ptr() as *const GLvoid,
                gl::DYNAMIC_DRAW,
            );

            // The VBO that will be associated with the vertex attribute at index 1, which does not
            // change throughout the lifetime of the program (thus, we use the flag `STATIC_DRAW` below).
            gl::CreateBuffers(1, &mut self.vbo_slice_colors);
            gl::NamedBufferData(
                self.vbo_slice_colors,
                colors_size as isize,
                colors.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            // Items that will be written to on the GPU (more or less every frame).
            // ...

            // The SSBO of slice vertices that will be written to whenever the slicing hyperplane moves.
            gl::CreateBuffers(1, &mut self.ssbo_slice_vertices);
            gl::NamedBufferData(
                self.ssbo_slice_vertices,
                vertices_size as isize,
                ptr::null() as *const GLvoid,
                gl::DYNAMIC_DRAW,
            );

            // The SSBO of slice triangle indices that will be written to whenever the slicing hyperplane moves.
            let indices_size = mem::size_of::<u32>() * 4usize * total_tetrahedra as usize;
            gl::CreateBuffers(1, &mut self.ssbo_slice_indices);
            gl::NamedBufferData(
                self.ssbo_slice_indices,
                indices_size as isize,
                ptr::null() as *const GLvoid,
                gl::DYNAMIC_DRAW,
            );

            // Set up SSBO bind points.
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, self.ssbo_tetrahedra);
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, self.ssbo_slice_vertices);
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, self.ssbo_slice_indices);
        }
    }

    pub fn slice(&mut self, rot: &Matrix4<f32>, hyperplane: &Hyperplane) {
        self.slice_program.bind();
        self.slice_program.uniform_4f("u_hyperplane_normal", &hyperplane.normal);
        self.slice_program.uniform_1f("u_hyperplane_displacement", hyperplane.displacement);
        self.slice_program.uniform_matrix_4f("u_rotation", rot);

        unsafe {

            // TODO: find the optimal value to launch here.
            gl::DispatchCompute(3240, 1, 1);
            gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);

        }

        self.slice_program.unbind();
    }

    /// Performs of a tetrahedral decomposition of the polytope.
    pub fn tetrahedralize(&mut self) {
        let mut tetrahedrons = Vec::new();

        for (cell_index, plane_and_faces) in self.gather_cells().iter().enumerate() {
            let mut prev_len = tetrahedrons.len();

            // The vertex that all tetrahedrons making up this solid will connect to.
            let mut apex = Vector4::from_value(f32::MAX);
            let (hyperplane, face_indices) = plane_and_faces;

            // Calculate the centroid of this cell, which is the average of all face centroids.
            let cell_centroid = utilities::average(
                &face_indices
                    .iter()
                    .map(|index| {
                        utilities::average(&self.get_vertices_for_face(*index), &Vector4::zero())
                    })
                    .collect::<Vec<_>>(),
                &Vector4::zero(),
            );

            // Iterate over each face of the current cell.
            for face_index in face_indices {
                // Get the vertices that make up this face.
                let face_vertices = self.get_vertices_for_face(*face_index);

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
            }

            println!(
                "{} tetrahedrons found for solid {}",
                tetrahedrons.len() - prev_len,
                cell_index
            );
        }

        println!(
            "Mesh tetrahedralization resulted in {} tetrahedrons.",
            tetrahedrons.len()
        );

        self.tetrahedrons = tetrahedrons;
    }
}
