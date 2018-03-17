#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_must_use)]
#![allow(unused_assignments)]
#![allow(unreachable_code)]
extern crate cgmath;
extern crate gl;
extern crate glutin;
extern crate image;

mod program;

use program::Program;

use std::mem;
use std::ptr;
use std::os::raw::c_void;
use std::time::{Duration, SystemTime};
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::str;

use gl::types::*;
use glutin::GlContext;
use cgmath::{InnerSpace, Matrix2, Matrix3, Matrix4, Perspective, Point2, Point3, SquareMatrix,
             Transform, Vector2, Vector3, Vector4, Zero};
use image::{GenericImage, ImageBuffer};

fn clear() {
    unsafe {
        gl::ClearColor(0.1, 0.05, 0.05, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}

struct FourCamera {
    from: Vector4<f32>,
    to: Vector4<f32>,
    up: Vector4<f32>,
    over: Vector4<f32>,
    look_at: Matrix4<f32>,
}

impl FourCamera {
    fn new(
        from: Vector4<f32>,
        to: Vector4<f32>,
        up: Vector4<f32>,
        over: Vector4<f32>,
    ) -> FourCamera {
        let mut cam = FourCamera {
            from,
            to,
            up,
            over,
            look_at: Matrix4::identity(),
        };
        cam.build_look_at();

        cam
    }

    /// Takes a 4D cross product between `u`, `v`, and `w`.
    fn cross(&self, u: &Vector4<f32>, v: &Vector4<f32>, w: &Vector4<f32>) -> Vector4<f32> {
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

    fn build_look_at(&mut self) {
        let wd = (self.to - self.from).normalize();
        let wa = self.cross(&self.up, &self.over, &wd).normalize();
        let wb = self.cross(&self.over, &wd, &wa).normalize();
        let wc = self.cross(&wd, &wa, &wb);

        self.look_at = Matrix4::from_cols(wa, wb, wc, wd);
    }
}

enum Plane {
    XY,
    YZ,
    ZX,
    XW,
    YW,
    ZW,
}

/// The 4D equivalent of a quaternion is known as a rotor.
/// https://math.stackexchange.com/questions/1402362/rotation-in-4d
fn get_simple_rotation_matrix(plane: Plane, angle: f32) -> Matrix4<f32> {
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
fn get_double_rotation_matrix(alpha: f32, beta: f32) -> Matrix4<f32> {
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

fn project(points: &Vec<f32>, cam: &FourCamera, t: f32) -> (Vec<f32>, Vec<f32>) {
    let mut projected = Vec::new();
    let mut depth_cue = Vec::new();

    let four_view = cam.look_at;
    //let rot_a = get_simple_rotation_matrix(Plane::YW, t);
    //let rot_b = get_simple_rotation_matrix(Plane::XW, t);
    let rot = get_double_rotation_matrix(t, t);

    for chunk in points.chunks(4) {
        let pt = Vector4::new(chunk[0], chunk[1], chunk[2], chunk[3]);

        let t = 1.0f32 / (std::f32::consts::PI / 4.0 * 0.5f32).tan();
        let temp = rot * (pt) - cam.from;
        let s = t / temp.dot(four_view.w);

        depth_cue.push(s);

        projected.extend_from_slice(&[
            s * temp.dot(four_view.x),
            s * temp.dot(four_view.y),
            s * temp.dot(four_view.z),
            1.0,
        ]);
    }
    (projected, depth_cue)
}

/// Counts the number of bits that `a` and `b` have in common. Processes
/// at least `bits`.
///
/// Reference: `https://stackoverflow.com/questions/28258882/number-of-digits-common-between-2-binary-numbers`
fn common_bits(a: u32, b: u32, bits: u32) -> u32 {
    if bits == 0 {
        return 0;
    }
    ((a & 1) == (b & 1)) as u32 + common_bits(a / 2, b / 2, bits - 1)
}

/// Loads the shape file at the specified `path` and returns a tuple of vectors.
/// The first vector will contain the vertex data and the second vector will
/// contain the edge indices.
fn load_shape(path: &Path) -> (Vec<f32>, Vec<u32>) {
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

    // TODO:
    //    for entry in &reader.lines().take(num_vertices) {
    //        for coordinate in entry.unwrap().split_whitespace() {
    //            vertices.push(coordinate.trim().parse().unwrap());
    //        }
    //    }

    println!(
        "Loaded file with {} vertices and {} edges",
        vertices.len(),
        edges.len()
    );

    (vertices, edges)
}

/// Generates a hypercube procedurally and returns a tuple of vectors.
/// The first vector will contain the vertex data and the second vector will
/// contain the edge indices.
///
/// Reference: `http://www.math.caltech.edu/~2014-15/2term/ma006b/05%20connectivity%201.pdf`
fn hypercube() -> (Vec<f32>, Vec<u32>) {
    // Two vertices are adjacent if they have `d - 1`
    // common coordinates.
    let d = 4;
    let adj = d - 1;
    let num_verts = 2u32.pow(d);
    let num_edges = 2u32.pow(d - 1) * d;
    println!(
        "Generating a hypercube with {} vertices and {} edges.",
        num_verts, num_edges
    );

    let mut vertices = Vec::with_capacity(num_verts as usize);
    let mut indices = Vec::with_capacity(num_edges as usize);

    for i in 0..num_verts {
        let mut num = i;

        // Generate vertices.
        for bit in 0..d {
            vertices.insert(0, (num & 1) as f32 * 2.0 - 1.0);
            num = num >> 1;
        }

        // Generate indices.
        for j in 0..num_verts {
            if i != j && common_bits(i, j, d) == adj {
                indices.push(i);
                indices.push(j);
            }
        }
    }

    (vertices, indices)
}

/// Generates an OpenGL shader program based on the source files specified by
/// `vs_path` (vertex shader) and `fs_path` (fragment shader).
fn load_shaders(vs_path: &Path, fs_path: &Path) -> Program {
    let mut vs = File::open(vs_path).expect("File not found");
    let mut fs = File::open(fs_path).expect("File not found");

    let mut vs_src = String::new();
    let mut fs_src = String::new();
    vs.read_to_string(&mut vs_src)
        .expect("Something went wrong reading the file");
    fs.read_to_string(&mut fs_src)
        .expect("Something went wrong reading the file");

    Program::new(vs_src, fs_src).unwrap()
}

/// Saves the current frame to disk at `path` with dimensions `w`x`h`.
fn save_frame(path: &Path, w: u32, h: u32) {
    let len = w * h * 3;
    let mut pixels: Vec<u8> = Vec::new();
    pixels.reserve(len as usize);

    unsafe {
        // We don't want any alignment padding on pixel rows.
        gl::PixelStorei(gl::PACK_ALIGNMENT, 1);
        gl::ReadPixels(
            0,
            0,
            w as i32,
            h as i32,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            pixels.as_mut_ptr() as *mut c_void,
        );
        pixels.set_len(len as usize);
    }

    image::save_buffer(path, &pixels, w, h, image::RGB(8)).unwrap();
}

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_dimensions(600, 600)
        .with_title("four");
    let context = glutin::ContextBuilder::new().with_multisampling(8);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();
    unsafe { gl_window.make_current() }.unwrap();
    gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

    // Set up the 4D shape(s).
    let shape_path = Path::new("shapes/120cell.txt");
    let (vertices, indices) = load_shape(&shape_path);

    // Set up the scene cameras.
    let mut four_cam = FourCamera::new(
        Vector4::new(2.83, 2.83, 0.01, 0.0),
        Vector4::zero(),
        Vector4::new(-0.71, 0.71, 0.0, 0.0),
        Vector4::new(0.0, 0.0, 1.0, 0.02),
    );

    let three_view = Matrix4::look_at(
        Point3::new(2.4, 0.99, 1.82),
        Point3::new(0.0, 0.0, 0.0),
        Vector3::unit_y(),
    );
    let three_projection =
        cgmath::perspective(cgmath::Rad(std::f32::consts::FRAC_PI_2), 1.0, 0.1, 1000.0);

    let program = load_shaders(
        Path::new("shaders/shader.vert"),
        Path::new("shaders/shader.frag"),
    );

    program.bind();
    program.uniform_4f("u_four_from", &four_cam.from);
    program.uniform_matrix_4f("u_four_view", &four_cam.look_at);
    program.uniform_matrix_4f("u_three_view", &three_view);
    program.uniform_matrix_4f("u_three_projection", &three_projection);

    let mut vao = 0;
    let mut vbo_position = 0;
    let mut vbo_depth = 0;
    let mut ebo = 0;
    unsafe {
        gl::Enable(gl::VERTEX_PROGRAM_POINT_SIZE);

        // Create the vertex array object.
        gl::CreateVertexArrays(1, &mut vao);

        let mut size = (vertices.len() * mem::size_of::<f32>()) as GLsizeiptr;

        // Create the vertex buffer for holding position data.
        gl::CreateBuffers(1, &mut vbo_position);
        gl::NamedBufferData(
            vbo_position,
            size,
            vertices.as_ptr() as *const GLvoid,
            gl::DYNAMIC_DRAW,
        );

        // Create the vertex buffer for holding depth cue data.
        size = (vertices.len() / 4usize * mem::size_of::<f32>()) as GLsizeiptr;
        gl::CreateBuffers(1, &mut vbo_depth);
        gl::NamedBufferData(vbo_depth, size, ptr::null(), gl::DYNAMIC_DRAW);

        // Create the index buffer.
        size = (indices.len() * mem::size_of::<u32>()) as GLsizeiptr;
        gl::CreateBuffers(1, &mut ebo);
        gl::NamedBufferData(
            ebo,
            size,
            indices.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW,
        );

        // Set up vertex attributes.
        let binding = 0;

        gl::EnableVertexArrayAttrib(vao, 0);
        gl::EnableVertexArrayAttrib(vao, 1);

        gl::VertexArrayAttribFormat(vao, 0, 4, gl::FLOAT, gl::FALSE, 0);
        gl::VertexArrayAttribFormat(vao, 1, 1, gl::FLOAT, gl::FALSE, 0);

        gl::VertexArrayAttribBinding(vao, 0, binding);
        gl::VertexArrayAttribBinding(vao, 1, binding + 1);

        gl::VertexArrayElementBuffer(vao, ebo);

        // Link vertex buffers to vertex attributes, via binding points.
        let offset = 0;
        gl::VertexArrayVertexBuffer(
            vao,
            binding,
            vbo_position,
            offset,
            (mem::size_of::<f32>() * 4 as usize) as i32,
        );
        gl::VertexArrayVertexBuffer(
            vao,
            binding + 1,
            vbo_depth,
            offset,
            mem::size_of::<f32>() as i32,
        );
    }

    let start = SystemTime::now();
    let mut cursor = Vector2::zero();

    loop {
        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::Closed => (),
                glutin::WindowEvent::MouseMoved { position, .. } => {
                    cursor.x = position.0 as f32 / 600.0;
                    cursor.y = position.1 as f32 / 600.0;
                }
                glutin::WindowEvent::KeyboardInput { input, .. } => {
                    if let glutin::ElementState::Pressed = input.state {
                        if let Some(key) = input.virtual_keycode {
                            if let glutin::VirtualKeyCode::S = key {
                                let path = Path::new("frame.png");
                                save_frame(path, 600, 600);
                            }
                        }
                    }
                }
                _ => (),
            },
            _ => (),
        });

        // Retrieve the number of milliseconds since application launch.
        let elapsed = start.elapsed().unwrap();
        let seconds = elapsed.as_secs() * 1000 + elapsed.subsec_nanos() as u64 / 1_000_000;
        let milliseconds = (seconds as f32) / 1000.0;

        let a = Vector4::new(0.0, 0.0, 0.0, 3.0);
        let b = Vector4::new(0.0, 0.0, 0.0, 4.0);
        let c = Vector4::new(1.0, 0.0, 0.0, 0.0);
        let d = Vector4::new(0.0, 0.0, 1.0, 0.0);
        four_cam.from = a.lerp(b, cursor.x);
        four_cam.over = c.lerp(d, cursor.y);
        four_cam.build_look_at();

        // Project the points from 4D -> 3D.
        let (projected, depth_cue) = project(&vertices, &four_cam, milliseconds);

        // Update GPU-side buffers.
        unsafe {
            let mut size = (projected.len() * mem::size_of::<f32>()) as GLsizeiptr;

            gl::NamedBufferSubData(vbo_position, 0, size, projected.as_ptr() as *const GLvoid);

            size = (depth_cue.len() * mem::size_of::<f32>()) as GLsizeiptr;

            gl::NamedBufferSubData(vbo_depth, 0, size, depth_cue.as_ptr() as *const GLvoid);
        }

        clear();

        unsafe {
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::POINTS, 0, (projected.len() / 4) as i32);
            gl::DrawElements(
                gl::LINES,
                indices.len() as i32,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
        }

        gl_window.swap_buffers().unwrap();
    }
}
