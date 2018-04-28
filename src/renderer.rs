use std::mem;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr;

use cgmath::{self, Vector4};
use gl;
use gl::types::*;

use tetrahedron::Tetrahedron;

pub struct Renderer {
    vao: u32,
    vbo: u32,
    ebo: u32,
}

impl Renderer {
    pub fn new() -> Renderer {
        let mut renderer = Renderer {
            vao: 0,
            vbo: 0,
            ebo: 0,
        };

        renderer.init();
        renderer
    }

    fn init(&mut self) {
        unsafe {
            gl::CreateVertexArrays(1, &mut self.vao);

            let mut size = (256 * mem::size_of::<f32>()) as GLsizeiptr;
            gl::CreateBuffers(1, &mut self.vbo);
            gl::NamedBufferData(
                self.vbo,
                size,
                ptr::null() as *const GLvoid,
                gl::DYNAMIC_DRAW,
            );

            size = (256 * mem::size_of::<u32>()) as GLsizeiptr;
            gl::CreateBuffers(1, &mut self.ebo);
            gl::NamedBufferData(
                self.ebo,
                size,
                Tetrahedron::get_edge_indices().as_ptr() as *const GLvoid,
                gl::DYNAMIC_DRAW,
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

    pub fn draw_tetrahedron(&self, tetra: &Tetrahedron) {
        unsafe {
            let transformed_vertices = tetra.get_transformed_vertices();

            // Each tetrahedron has 4 vertices, each of which has 4 components.
            let vbo_upload_size = (4 * 4 * mem::size_of::<GLfloat>()) as GLsizeiptr;
            gl::NamedBufferSubData(
                self.vbo,
                0,
                vbo_upload_size,
                transformed_vertices.as_ptr() as *const c_void,
            );

            let edges = Tetrahedron::get_edge_indices();
            let ebo_upload_size = (edges.len() * mem::size_of::<u32>()) as GLsizeiptr;
            gl::NamedBufferSubData(
                self.ebo,
                0,
                ebo_upload_size,
                edges.as_ptr() as *const GLvoid,
            );

            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::LINES, 6 * 2 as i32, gl::UNSIGNED_INT, ptr::null());

            //gl::DrawArrays(gl::POINTS, 0, transformed_vertices.len() as i32);
        }
    }

    pub fn draw_tetrahedron_slice(&self, slice_vertices: &Vec<Vector4<f32>>) {
        unsafe {
            const COMPONENTS_PER_VERTEX: usize = 4;
            const NUMBER_OF_INDICES: usize = 6;
            let slice_indices = Tetrahedron::get_quad_indices();

            // Each tetrahedron has 4 vertices, each of which has 4 components.
            let vbo_upload_size = (COMPONENTS_PER_VERTEX * slice_vertices.len()
                * mem::size_of::<GLfloat>()) as GLsizeiptr;
            gl::NamedBufferSubData(
                self.vbo,
                0,
                vbo_upload_size,
                slice_vertices.as_ptr() as *const c_void,
            );

            let ebo_upload_size = (NUMBER_OF_INDICES * mem::size_of::<u32>()) as GLsizeiptr;
            gl::NamedBufferSubData(
                self.ebo,
                0,
                ebo_upload_size,
                slice_indices.as_ptr() as *const GLvoid,
            );

            // First, draw each vertex of the slice as a point.
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::POINTS, 0, slice_vertices.len() as i32);

            // Then, draw the triangle or quadrilateral, depending on the number of vertices passed
            // to this function.
            let number_of_elements = match slice_vertices.len() {
                3 => 3,
                4 => 6,
                _ => 0,
            };
            gl::DrawElements(
                gl::TRIANGLES,
                number_of_elements as i32,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
        }
    }
}
