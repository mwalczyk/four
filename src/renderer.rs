use std::mem;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr;

use gl;
use gl::types::*;

use tetrahedron::{Tetrahedron, TetrahedronSlice, INDICES};

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
                INDICES.as_ptr() as *const GLvoid,
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

    pub fn draw_tetrahedron(&self, tetra: &Tetrahedron) {
        unsafe {
            // Each tetrahedron has 4 vertices, each of which has 4 components.
            let vbo_upload_size = (4 * 4 * mem::size_of::<GLfloat>()) as GLsizeiptr;
            gl::NamedBufferSubData(self.vbo, 0, vbo_upload_size, tetra.vertices.as_ptr() as *const c_void);

            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::LINES, 6 * 2 as i32, gl::UNSIGNED_INT, ptr::null());
        }
    }

    pub fn draw_tetrahedron_slice(&self, tetra_slice: &TetrahedronSlice) {
        // TODO
    }
}
