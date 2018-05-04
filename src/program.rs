use gl;
use gl::types::*;

use cgmath;
use cgmath::{Array, Matrix, Matrix4, Vector2, Vector3, Vector4};

use std::ffi::CString;
use std::ptr;
use std::str;

pub struct UniformEntry {
    name: String,
    location: GLint,
}

pub struct Program {
    pub id: GLuint,
}

impl Program {
    /// Compiles a shader of type `stage` from the source held in `src`.
    fn compile_shader(src: &String, stage: GLenum) -> Result<GLuint, String> {
        let shader;
        unsafe {
            shader = gl::CreateShader(stage);

            // Attempt to compile the shader.
            let c_str = CString::new(src.as_bytes()).unwrap();
            gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
            gl::CompileShader(shader);

            // Get the compile status.
            let mut status = gl::FALSE as GLint;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

            // Fail on error
            if status != (gl::TRUE as GLint) {
                let mut len = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = Vec::with_capacity(len as usize);

                // Subtract 1 to skip the trailing null character.
                buffer.set_len((len as usize) - 1);

                gl::GetShaderInfoLog(
                    shader,
                    len,
                    ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut GLchar,
                );

                let error = String::from_utf8(buffer)
                    .ok()
                    .expect("ShaderInfoLog not valid utf8");
                return Err(error);
            }
        }

        Ok(shader)
    }

    fn link_single_stage_program(cs: GLuint) -> Result<GLuint, String> {
        unsafe {
            let program = gl::CreateProgram();
            gl::AttachShader(program, cs);
            gl::LinkProgram(program);

            // Get the link status.
            let mut status = gl::FALSE as GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

            // If there was an error, return the error string.
            if status != (gl::TRUE as GLint) {
                let mut len: GLint = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = Vec::with_capacity(len as usize);

                // Subtract 1 to skip the trailing null character.
                buffer.set_len((len as usize) - 1);

                gl::GetProgramInfoLog(
                    program,
                    len,
                    ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut GLchar,
                );
                gl::DeleteShader(cs);

                let error = String::from_utf8(buffer)
                    .ok()
                    .expect("ProgramInfoLog not valid utf8");
                return Err(error);
            }

            Ok(program)
        }
    }

    fn link_two_stage_program(vs: GLuint, fs: GLuint) -> Result<GLuint, String> {
        unsafe {
            let program = gl::CreateProgram();
            gl::AttachShader(program, vs);
            gl::AttachShader(program, fs);
            gl::LinkProgram(program);

            // Get the link status.
            let mut status = gl::FALSE as GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

            // If there was an error, return the error string.
            if status != (gl::TRUE as GLint) {
                let mut len: GLint = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = Vec::with_capacity(len as usize);

                // Subtract 1 to skip the trailing null character.
                buffer.set_len((len as usize) - 1);

                gl::GetProgramInfoLog(
                    program,
                    len,
                    ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut GLchar,
                );
                gl::DeleteShader(fs);
                gl::DeleteShader(vs);

                let error = String::from_utf8(buffer)
                    .ok()
                    .expect("ProgramInfoLog not valid utf8");
                return Err(error);
            }

            Ok(program)
        }
    }

    fn perform_reflection(src: &str) {}

    pub fn two_stage(vs_src: String, fs_src: String) -> Option<Program> {
        // Make sure that compiling each of the shaders was successful.
        let compile_vs_res = Program::compile_shader(&vs_src, gl::VERTEX_SHADER);
        let compile_fs_res = Program::compile_shader(&fs_src, gl::FRAGMENT_SHADER);

        match (compile_vs_res, compile_fs_res) {
            (Ok(vs_id), Ok(fs_id)) => {
                // Make sure that linking the shader program was successful.
                if let Ok(id) = Program::link_two_stage_program(vs_id, fs_id) {
                    // If everything went ok, return the shader program.
                    return Some(Program { id });
                } else {
                    return None;
                }
            }
            // Both shader stages resulted in an error.
            (Err(vs_err), Err(fs_err)) => {
                println!("{}", vs_err);
                println!("{}", fs_err);
                return None;
            }
            // The vertex shader resulted in an error.
            (Err(vs_err), Ok(_)) => {
                println!("{}", vs_err);
                return None;
            }
            // The fragment shader resulted in an error.
            (Ok(_), Err(fs_err)) => {
                println!("{}", fs_err);
                return None;
            }
        }
    }

    pub fn single_stage(cs_src: String) -> Option<Program> {
        let compile_cs_res = Program::compile_shader(&cs_src, gl::COMPUTE_SHADER);

        match compile_cs_res {
            Ok(cs_id) => {
                if let Ok(id) = Program::link_single_stage_program(cs_id) {
                    return Some(Program { id });
                } else {
                    return None;
                }
            },
            Err(cs_err) => {
                println!("{}", cs_err);
                return None;
            }
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::UseProgram(0);
        }
    }

    pub fn uniform_1i(&self, name: &str, value: i32) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr());
            gl::ProgramUniform1i(self.id, location, value as gl::types::GLint);
        }
    }

    pub fn uniform_1ui(&self, name: &str, value: u32) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr());
            gl::ProgramUniform1ui(self.id, location, value as gl::types::GLuint);
        }
    }

    pub fn uniform_1f(&self, name: &str, value: f32) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr());
            gl::ProgramUniform1f(self.id, location, value as gl::types::GLfloat);
        }
    }

    pub fn uniform_2f(&self, name: &str, value: &cgmath::Vector2<f32>) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr());
            gl::ProgramUniform2fv(self.id, location, 1, value.as_ptr());
        }
    }

    pub fn uniform_3f(&self, name: &str, value: &cgmath::Vector3<f32>) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr());
            gl::ProgramUniform3fv(self.id, location, 1, value.as_ptr());
        }
    }

    pub fn uniform_4f(&self, name: &str, value: &cgmath::Vector4<f32>) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr());
            gl::ProgramUniform4fv(self.id, location, 1, value.as_ptr());
        }
    }

    pub fn uniform_matrix_4f(&self, name: &str, value: &cgmath::Matrix4<f32>) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr());
            gl::ProgramUniformMatrix4fv(self.id, location, 1, gl::FALSE, value.as_ptr());
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}
