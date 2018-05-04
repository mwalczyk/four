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

            let mut size = (1024 * mem::size_of::<f32>()) as GLsizeiptr;
            gl::CreateBuffers(1, &mut self.vbo);
            gl::NamedBufferData(
                self.vbo,
                size,
                ptr::null() as *const GLvoid,
                gl::DYNAMIC_DRAW,
            );

            size = (1024 * mem::size_of::<u32>()) as GLsizeiptr;
            gl::CreateBuffers(1, &mut self.ebo);
            gl::NamedBufferData(
                self.ebo,
                size,
                Tetrahedron::get_edge_indices().as_ptr() as *const GLvoid,
                gl::DYNAMIC_DRAW,
            );

            gl::EnableVertexArrayAttrib(self.vao, 0);
            gl::VertexArrayAttribFormat(self.vao, 0, 4, gl::FLOAT, gl::FALSE, 0);
            gl::VertexArrayAttribBinding(self.vao, 0, 0);
            gl::VertexArrayElementBuffer(self.vao, self.ebo);

            gl::VertexArrayVertexBuffer(
                self.vao,
                0,
                self.vbo,
                0,
                (mem::size_of::<f32>() * 4 as usize) as i32,
            );
        }
    }

    pub fn draw_tetrahedron(&self, tetra: &Tetrahedron) {
        unsafe {
            // Each tetrahedron has 4 vertices, each of which has 4 components.
            let vbo_upload_size = (mem::size_of::<Vector4<f32>>() * 4) as GLsizeiptr;
            gl::NamedBufferSubData(
                self.vbo,
                0,
                vbo_upload_size,
                tetra.vertices.as_ptr() as *const c_void,
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
            gl::DrawArrays(gl::POINTS, 0, 4);
        }
    }
}
