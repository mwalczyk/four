use std::f32;
use std::mem;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr;

use cgmath::{self, Array, InnerSpace, Matrix4, SquareMatrix, Vector4, Zero};
use gl;
use gl::types::*;

use hyperplane::Hyperplane;
use math;
use polychora::{Definition, Polychoron};
use program::Program;
use tetrahedron::Tetrahedron;
use utilities;

/// A struct representing an entry in the indirect draw buffer.
#[repr(C)]
struct DrawCommand {
    count: u32,
    instance_count: u32,
    first: u32,
    base_instance: u32,
}

/// A 4-dimensional mesh.
pub struct Mesh {
    /// The vertices of the 4-dimensional mesh.
    vertices: Vec<Vector4<f32>>,

    /// The edges of the 4-dimensional mesh.
    edges: Vec<u32>,

    /// The faces of the 4-dimensional mesh.
    faces: Vec<u32>,

    /// The type of polychoron that this mesh represents.
    polychoron: Polychoron,

    /// The topology (definition) of the polychoron that this mesh represents.
    def: Definition,

    /// A list of tetrahedra (embedded in 4-dimensions) that make up this mesh.
    tetrahedra: Vec<Tetrahedron>,

    /// The current transform (translation, rotation, scale) of this mesh (in 4-dimensions).
    transform: Matrix4<f32>,

    /// The compute shader that is used to compute 3-dimensional slices of this mesh.
    compute: Program,

    /// The vertex array object (VAO) that is used for drawing a 3D slice of this mesh.
    vao_slice: u32,

    /// A GPU-side buffer that contains all of the tetrahedra that make up this mesh.
    buffer_tetrahedra: u32,

    /// A GPU-side buffer that contains all of the colors used to render 3-dimensional slices of this mesh.
    buffer_slice_colors: u32,

    /// A GPU-side buffer that contains all of the vertices that make up the active 3-dimensional cross-section of this mesh.
    buffer_slice_vertices: u32,

    /// A GPU-side buffer that will be filled with indirect drawing commands via the `compute` program.
    buffer_indirect_commands: u32,

    /// The VAO that is used for drawing all of the tetrahedra that make up this mesh.
    vao_tetrahedra: u32,

    /// The EBO that is used for drawing all of the edges of the tetrahedra that make up this mesh.
    ebo_tetrahedra: u32,

    /// The VAO that is used for drawing the wireframe of this polychoron.
    vao_edges: u32,

    /// A GPU-side buffer that contains all of the unique vertices that make up this polychoron.
    vbo_edges: u32,

    /// The EBO that is used for drawing the wireframe of this polychoron.
    ebo_edges: u32,
}

impl Mesh {
    pub fn new(polychoron: Polychoron) -> Mesh {
        match polychoron {
            Polychoron::Cell24Rectified => eprintln!(
                "Drawing of this shape is not yet supported - please try another polychoron"
            ),
            _ => (),
        }

        let compute = utilities::load_file_as_string(Path::new("shaders/compute_slice.glsl"));

        let mut mesh = Mesh {
            vertices: polychoron.get_vertices(),
            edges: polychoron.get_edges(),
            faces: polychoron.get_faces(),
            polychoron,
            def: polychoron.get_definition(),
            tetrahedra: Vec::new(),
            transform: Matrix4::identity(),
            compute: Program::single_stage(compute).unwrap(),
            vao_slice: 0,
            buffer_tetrahedra: 0,
            buffer_slice_colors: 0,
            buffer_slice_vertices: 0,
            buffer_indirect_commands: 0,
            vao_tetrahedra: 0,
            ebo_tetrahedra: 0,
            vao_edges: 0,
            vbo_edges: 0,
            ebo_edges: 0,
        };

        mesh.tetrahedralize();
        mesh.init_render_objects();
        mesh
    }

    /// Returns an array of all of the tetrahedra that make up this mesh.
    pub fn get_tetrahedra(&self) -> &Vec<Tetrahedron> {
        &self.tetrahedra
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

    /// Returns the `i`th vertex of this mesh.
    pub fn get_vertex(&self, i: u32) -> Vector4<f32> {
        self.vertices[i as usize]
    }

    /// Returns an unordered tuple of the two vertices that make up the `i`th
    /// edge of this mesh.
    pub fn get_vertices_for_edge(&self, i: u32) -> (Vector4<f32>, Vector4<f32>) {
        let idx_edge_s = (i * self.def.vertices_per_edge) as usize;
        let idx_edge_e = (i * self.def.vertices_per_edge + self.def.vertices_per_edge) as usize;
        let pair = &self.edges[idx_edge_s..idx_edge_e];

        (self.get_vertex(pair[0]), self.get_vertex(pair[1]))
    }

    /// Returns an unordered list of the unique vertices that make up the `i`th
    /// face of this mesh.
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

    /// Set this mesh's current transform (in 4-dimensions). This will affect how the
    /// mesh is sliced.
    pub fn set_transform(&mut self, transform: &Matrix4<f32>) {
        self.transform = *transform;
    }

    /// Slice this mesh with a 4-dimensional `hyperplane`.
    pub fn slice(&mut self, hyperplane: &Hyperplane) {
        self.compute.bind();
        self.compute
            .uniform_4f("u_hyperplane_normal", &hyperplane.normal);
        self.compute
            .uniform_1f("u_hyperplane_displacement", hyperplane.displacement);

        self.compute
            .uniform_matrix_4f("u_transform", &self.transform);

        unsafe {
            // Bind buffers for read / write.
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, self.buffer_tetrahedra);
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, self.buffer_slice_vertices);
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, self.buffer_indirect_commands);

            let dispatch = (self.tetrahedra.len() as f32 / 128.0).ceil();
            gl::DispatchCompute(dispatch as u32, 1, 1);

            // Barrier against subsequent SSBO reads and indirect drawing commands.
            gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT | gl::COMMAND_BARRIER_BIT);
        }

        self.compute.unbind();
    }

    /// Draws a 3-dimensional slice of the 4-dimensional mesh.
    pub fn draw_slice(&self) {
        unsafe {
            gl::BindVertexArray(self.vao_slice);

            // Bind the buffer that contains indirect draw commands.
            gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, self.buffer_indirect_commands);

            // Dispatch indirect draw commands.
            gl::MultiDrawArraysIndirect(
                gl::TRIANGLES,
                ptr::null() as *const GLvoid,
                self.tetrahedra.len() as i32,
                mem::size_of::<DrawCommand>() as i32,
            );
        }
    }

    /// Draws a 3-dimensional projection of the 4-dimensional tetrahedra that make up this
    /// mesh.
    pub fn draw_tetrahedra(&self) {
        unsafe {
            let number_of_tetrahedral_edges =
                self.tetrahedra.len() * Tetrahedron::get_number_of_edges() * 2;

            gl::BindVertexArray(self.vao_tetrahedra);
            gl::DrawElements(
                gl::LINES,
                number_of_tetrahedral_edges as i32,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
        }
    }

    /// Draws a 3-dimensional projection of the skeleton (wireframe) of this polychoron.
    pub fn draw_edges(&self) {
        unsafe {
            gl::BindVertexArray(self.vao_edges);
            gl::DrawElements(
                gl::LINES,
                self.edges.len() as i32,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
        }
    }

    /// Given the H-representation of this polychoron, return a list of lists, where
    /// each sub-list contains the indices of all faces that are inside the `i`th
    /// hyperplane.
    ///
    /// Here, we take a relatively brute-force approach by iterating over all of the faces
    /// of this polychoron. For the 120-cell, for example, there are 720 faces. For each
    /// face, we check if all of its vertices are "inside" of the current hyperplane. If so,
    /// we know that this face is part of the cell that is bounded by the current hyper-
    /// plane.
    ///
    /// The goal is to isolate which faces are part of which cells, since this information
    /// isn't part of any of the shape files.
    fn gather_cells(&self) -> Vec<(Hyperplane, Vec<u32>)> {
        let mut cells = Vec::new();

        for hyperplane in self.polychoron.get_h_representation().iter() {
            let mut faces_in_hyperplane = Vec::new();

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

            dbg!(format!(
                "{} faces found inside of the hyperplane: {:?}",
                faces_in_hyperplane.len(),
                hyperplane
            ));

            cells.push((*hyperplane, faces_in_hyperplane));
        }

        cells
    }

    /// Performs of a tetrahedral decomposition of the polychoron.
    ///
    /// Reference: `https://www.ics.uci.edu/~eppstein/projects/tetra/`
    fn tetrahedralize(&mut self) {
        let mut tetrahedrons = Vec::new();

        for (cell_index, plane_and_faces) in self.gather_cells().iter().enumerate() {
            let prev_len = tetrahedrons.len();

            // The vertex that all tetrahedrons making up this solid will connect to.
            let mut apex = Vector4::from_value(f32::MAX);
            let (hyperplane, face_indices) = plane_and_faces;

            // Calculate the centroid of this cell, which is the average of all face centroids.
            let cell_centroid = utilities::average(
                &face_indices
                    .iter()
                    .map(|index| {
                        let face_centroid = utilities::average(
                            &self.get_vertices_for_face(*index),
                            &Vector4::zero(),
                        );

                        face_centroid
                    })
                    .collect::<Vec<_>>(),
                &Vector4::zero(),
            );

            dbg!(format!(
                "Length of cell centroid: {}",
                cell_centroid.magnitude()
            ));

            // Iterate over each face of the current cell.
            for face_index in face_indices {
                // Get the vertices that make up this face.
                let face_vertices = self.get_vertices_for_face(*face_index);

                // First, we need to triangulate this face into several, non-overlapping
                // triangles.
                //
                // a -- b
                // |  / |
                // | /  |
                // c -- d
                //
                // We can do this by create a triangle fan, starting a one of the face
                // vertices. However, this assumes that our vertices are sorted in
                // some order (clockwise or counter-clockwise). So, the first thing we
                // do is, collect all of the face vertices and sort them.
                let face_vertices_sorted = math::sort_points_on_plane(&face_vertices, &hyperplane);

                if apex.x == f32::MAX {
                    apex = face_vertices[0];
                }

                // We only want to tetrahedralize faces that are NOT connected to the apex.
                if !face_vertices.contains(&apex) {
                    // Create a triangle fan, starting at the first vertex in the (sorted) list.
                    //
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
                            cell_index as u32,
                            cell_centroid,
                        ));
                    }
                }
            }

            dbg!(format!(
                "{} tetrahedrons found for cell at index: {}",
                tetrahedrons.len() - prev_len,
                cell_index
            ));
        }

        dbg!(format!(
            "Mesh tetrahedralization resulted in {} tetrahedrons.",
            tetrahedrons.len()
        ));

        self.tetrahedra = tetrahedrons;
    }

    /// Gathers all of the necessary vertex attributes required to render the tetrahedra
    /// that make up this mesh. In particular, this function returns the vertex positions
    /// and colors.
    fn gather_tetrahedra_attributes(&self) -> (Vec<Vector4<f32>>, Vec<Vector4<f32>>) {
        // Set up CPU-side buffers.
        let mut vertices = Vec::new();
        let mut colors = Vec::new();

        for tetra in self.tetrahedra.iter() {
            // First, push back all of this tetrahedron's vertices.
            vertices.extend_from_slice(tetra.get_vertices());

            // Any tetrahedral slice can have at most 6 vertices (a quadrilateral, 2 triangles).
            let max_vertices_per_slice = 6;

            // Next, push back all of this tetrahedron's colors (currently, we are
            // using the cell centroid to generate some sort of shading / colors).
            // TODO: see notes on attribute divisors in `init_slice_objects(...)`
            for i in 0..max_vertices_per_slice {
                colors.push(tetra.cell_centroid);
            }
        }

        (vertices, colors)
    }

    /// Gather all of the edge indices for the tetrahedra that make up this mesh.
    fn gather_tetrahedra_indices(&self) -> Vec<u32> {
        let mut indices = Vec::new();

        // Gather the base indices used for drawing a tetrahedron, i.e.
        // `[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]`, and apply
        // relative offsets.
        let local_indices = Tetrahedron::get_edge_indices();

        for (i, tetra) in self.tetrahedra.iter().enumerate() {
            // Generate a new set of edge indices for this tetrahedron.
            for (a, b) in local_indices.iter() {
                // Create a new set of indices to draw the current tetrahedron. First,
                // we add `4 * i`, since each tetrahedron has 4 vertices.
                let offset = (Tetrahedron::get_number_of_vertices() * i) as u32;

                indices.push(a + offset);
                indices.push(b + offset);
            }
        }

        indices
    }

    /// Initializes all OpenGL objects (VAOs, buffers, etc.): see functions below.
    fn init_render_objects(&mut self) {
        self.init_slice_objects();
        self.init_tetrahedra_objects();
        self.init_edges_objects();
    }

    /// Initializes all OpenGL objects for rendering a 3-dimensional slice of this
    /// 4-dimensional polychoron.
    fn init_slice_objects(&mut self) {
        unsafe {
            gl::CreateVertexArrays(1, &mut self.vao_slice);

            // Set up attribute #0: positions.
            const ATTR_POS: u32 = 0;
            const BINDING_POS: u32 = 0;
            gl::EnableVertexArrayAttrib(self.vao_slice, ATTR_POS);
            gl::VertexArrayAttribFormat(
                self.vao_slice,
                ATTR_POS,
                self.def.components_per_vertex as i32,
                gl::FLOAT,
                gl::FALSE,
                0,
            );
            gl::VertexArrayAttribBinding(self.vao_slice, ATTR_POS, BINDING_POS);

            // Set up attribute #1: colors.
            const ATTR_COL: u32 = 1;
            const BINDING_COL: u32 = 1;
            gl::EnableVertexArrayAttrib(self.vao_slice, ATTR_COL);
            gl::VertexArrayAttribFormat(
                self.vao_slice,
                ATTR_COL,
                self.def.components_per_vertex as i32,
                gl::FLOAT,
                gl::FALSE,
                0,
            );
            gl::VertexArrayAttribBinding(self.vao_slice, ATTR_COL, BINDING_COL);
            // TODO: gl::VertexArrayBindingDivisor(self.vao_slice, BINDING_COL, 6);

            let (vertices, colors) = self.gather_tetrahedra_attributes();

            // Any tetrahedral slice can have at most 6 vertices (a quadrilateral, 2 triangles).
            let max_vertices_per_slice = 6;

            let vertices_size = mem::size_of::<Vector4<f32>>()
                * Tetrahedron::get_number_of_vertices()
                * self.tetrahedra.len() as usize;
            let colors_size = mem::size_of::<Vector4<f32>>()
                * max_vertices_per_slice
                * self.tetrahedra.len() as usize;

            // The VBO that will be associated with the vertex attribute #1, which does not change
            // throughout the lifetime of the program (thus, we use the flag `STATIC_DRAW` below).
            gl::CreateBuffers(1, &mut self.buffer_slice_colors);
            gl::NamedBufferData(
                self.buffer_slice_colors,
                colors_size as isize,
                colors.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            // The buffer that will be bound at index #0 and read from.
            gl::CreateBuffers(1, &mut self.buffer_tetrahedra);
            gl::NamedBufferData(
                self.buffer_tetrahedra,
                vertices_size as isize,
                vertices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            // Items that will be written to on the GPU (more or less every frame).
            // ...

            // The buffer of slice vertices that will be written to whenever the slicing hyperplane moves.
            let mut alloc_size =
                mem::size_of::<Vector4<f32>>() * max_vertices_per_slice * self.tetrahedra.len();
            gl::CreateBuffers(1, &mut self.buffer_slice_vertices);
            gl::NamedBufferData(
                self.buffer_slice_vertices,
                alloc_size as isize,
                ptr::null() as *const GLvoid,
                gl::STREAM_DRAW,
            );

            // The buffer of draw commands that will be filled out by the compute shader dispatch.
            alloc_size = mem::size_of::<DrawCommand>() * self.tetrahedra.len();
            gl::CreateBuffers(1, &mut self.buffer_indirect_commands);
            gl::NamedBufferData(
                self.buffer_indirect_commands,
                alloc_size as isize,
                ptr::null() as *const GLvoid,
                gl::STREAM_DRAW,
            );

            // Set up SSBO bind points.
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, self.buffer_tetrahedra);
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, self.buffer_slice_vertices);
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, self.buffer_indirect_commands);

            // Setup vertex attribute bindings.
            gl::VertexArrayVertexBuffer(
                self.vao_slice,
                BINDING_POS,
                self.buffer_slice_vertices,
                0,
                mem::size_of::<Vector4<f32>>() as i32,
            );
            gl::VertexArrayVertexBuffer(
                self.vao_slice,
                BINDING_COL,
                self.buffer_slice_colors,
                0,
                mem::size_of::<Vector4<f32>>() as i32,
            );

            let mut local_size: [i32; 3] = [0; 3];
            gl::GetProgramiv(
                self.compute.get_id(),
                gl::COMPUTE_WORK_GROUP_SIZE,
                local_size.as_mut_ptr(),
            );
        }
    }

    /// Initializes all OpenGL objects for rendering wireframes of all of the
    /// tetrahedra that make up this polychoron, which are embedded in 4-dimensions.
    fn init_tetrahedra_objects(&mut self) {
        unsafe {
            // First, create the vertex array object.
            gl::CreateVertexArrays(1, &mut self.vao_tetrahedra);

            // Create the element buffer that will hold all of the edge indices for rendering
            // wireframes of all of the tetrahedra that make up this polychoron.
            let indices = self.gather_tetrahedra_indices();
            let indices_size = (indices.len() * mem::size_of::<u32>()) as GLsizeiptr;

            gl::CreateBuffers(1, &mut self.ebo_tetrahedra);
            gl::NamedBufferData(
                self.ebo_tetrahedra,
                indices_size,
                indices.as_ptr() as *const GLvoid,
                gl::DYNAMIC_DRAW,
            );

            gl::EnableVertexArrayAttrib(self.vao_tetrahedra, 0);
            gl::VertexArrayAttribFormat(
                self.vao_tetrahedra,
                0,
                self.def.components_per_vertex as i32,
                gl::FLOAT,
                gl::FALSE,
                0,
            );
            gl::VertexArrayAttribBinding(self.vao_tetrahedra, 0, 0);

            // Setup vertex attribute bindings: notice that we use the same VBO from above that
            // holds all of the vertices of the tetrahedra that make up this polychoron.
            gl::VertexArrayVertexBuffer(
                self.vao_tetrahedra,
                0,
                self.buffer_tetrahedra,
                0,
                (mem::size_of::<f32>() * self.def.components_per_vertex as usize) as i32,
            );

            // Bind the EBO to the VAO.
            gl::VertexArrayElementBuffer(self.vao_tetrahedra, self.ebo_tetrahedra);
        }
    }

    /// Initializes all OpenGL objects for rendering the edges of this polychoron.
    fn init_edges_objects(&mut self) {
        unsafe {
            // First, create the vertex array object.
            gl::CreateVertexArrays(1, &mut self.vao_edges);

            // Set up attribute #0: positions (for now, we ignore colors).
            const ATTR_POS: u32 = 0;
            const BINDING_POS: u32 = 0;
            gl::EnableVertexArrayAttrib(self.vao_edges, ATTR_POS);
            gl::VertexArrayAttribFormat(
                self.vao_edges,
                ATTR_POS,
                self.def.components_per_vertex as i32,
                gl::FLOAT,
                gl::FALSE,
                0,
            );
            gl::VertexArrayAttribBinding(self.vao_edges, ATTR_POS, BINDING_POS);

            // Create the vertex buffer that will hold all of the polychoron's unique vertices.
            let vertices_size =
                (self.vertices.len() * mem::size_of::<Vector4<f32>>()) as GLsizeiptr;

            gl::CreateBuffers(1, &mut self.vbo_edges);
            gl::NamedBufferData(
                self.vbo_edges,
                vertices_size as isize,
                self.vertices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            // Setup vertex attribute bindings.
            gl::VertexArrayVertexBuffer(
                self.vao_edges,
                BINDING_POS,
                self.vbo_edges,
                0,
                mem::size_of::<Vector4<f32>>() as i32,
            );

            // Create the element buffer that will hold all of the edge indices for rendering
            // a wireframe of this polychoron.
            let edges_size = (self.edges.len() * mem::size_of::<u32>()) as GLsizeiptr;

            gl::CreateBuffers(1, &mut self.ebo_edges);
            gl::NamedBufferData(
                self.ebo_edges,
                edges_size,
                self.edges.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            // Bind the EBO to the VAO.
            gl::VertexArrayElementBuffer(self.vao_edges, self.ebo_edges);
        }
    }
}
