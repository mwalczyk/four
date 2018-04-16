use std::mem;
use std::os::raw::c_void;
use std::ptr;

use gl;
use gl::types::*;

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

    pub fn draw(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);

            gl::DrawArrays(gl::POINTS, 0, (self.vertices.len() / 4) as i32);

            gl::DrawElements(
                gl::LINES,
                self.edges.len() as i32,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
        }
    }
}
