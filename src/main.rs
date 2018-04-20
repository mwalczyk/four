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

mod camera;
mod hyperplane;
mod polytope;
mod program;
mod renderer;
mod rotations;
mod tetrahedron;

use camera::Camera;
use hyperplane::Hyperplane;
use polytope::Polytope;
use program::Program;
use renderer::Renderer;
use tetrahedron::Tetrahedron;

use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::os::raw::c_void;
use std::path::Path;
use std::str;
use std::time::{Duration, SystemTime};

use cgmath::{InnerSpace, Matrix2, Matrix3, Matrix4, Perspective, Point2, Point3, Rotation,
             SquareMatrix, Transform, Vector2, Vector3, Vector4, Zero};
use glutin::GlContext;
use image::{GenericImage, ImageBuffer};

fn clear() {
    unsafe {
        gl::ClearColor(0.1, 0.05, 0.05, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
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

fn load_shapes() -> Vec<Polytope> {
    let mut polytopes = Vec::new();

    for entry in fs::read_dir("shapes").unwrap() {
        let path = entry.unwrap().path();
        let file = path.file_stem().unwrap();
        let ext = path.extension();

        if ext == Some(OsStr::new("txt")) {
            polytopes.push(Polytope::from_file(Path::new(&path)));
        }
    }
    polytopes
}

fn main() {
    const WIDTH: u32 = 600;
    const HEIGHT: u32 = 600;

    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_dimensions(WIDTH, HEIGHT)
        .with_title("four");
    let context = glutin::ContextBuilder::new().with_multisampling(8);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();
    unsafe { gl_window.make_current() }.unwrap();
    gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

    unsafe {
        // For now, we don't really know the winding order of the tetrahedron
        // slices, so we want to disable face culling.
        gl::Disable(gl::CULL_FACE);

        // Enable depth testing.
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);

        // Enable alpha blending.
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    // Set up the 4D shape(s).
    let mut polytopes = load_shapes();
    let tetrahedrons = polytopes[0].tetrahedralize();
    println!(
        "Mesh tetrahedralization resulted in {} tetrahedrons",
        tetrahedrons.len()
    );

    // Set up the scene cameras.
    let four_cam = Camera::new(
        Vector4::new(3.0, 0.0, 0.0, 0.0),
        Vector4::zero(),
        Vector4::new(0.0, 1.0, 0.0, 0.0),
        Vector4::new(0.0, 0.0, 1.0, 0.0),
    );
    let mut four_rotation = Matrix4::identity();

    let mut three_rotation = Matrix4::identity();
    let three_view = Matrix4::look_at(
        Point3::new(2.5, 0.0, 0.0),
        Point3::new(0.0, 0.0, 0.0),
        Vector3::unit_y(),
    );
    let three_projection =
        cgmath::perspective(cgmath::Rad(std::f32::consts::FRAC_PI_2), 1.0, 0.1, 1000.0);

    let program = load_shaders(
        Path::new("shaders/shader.vert"),
        Path::new("shaders/shader.frag"),
    );

    let renderer = Renderer::new();

    program.bind();

    let start = SystemTime::now();
    let mut cursor_prev = Vector2::zero();
    let mut cursor_curr = Vector2::zero();
    let mut cursor_pressed = Vector2::zero();
    let mut lmouse_pressed = false;
    let mut rmouse_pressed = false;
    let mut shift_pressed = false;
    let mut alt_pressed = false;
    let mut draw_index = 0;

    let mut hyperplane = Hyperplane::new(Vector4::new(1.0, 1.0, 1.0, 1.0), 0.1);

    loop {
        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::Closed => (),
                glutin::WindowEvent::MouseMoved { position, .. } => {
                    cursor_prev = cursor_curr;
                    cursor_curr.x = position.0 as f32 / WIDTH as f32;
                    cursor_curr.y = position.1 as f32 / HEIGHT as f32;
                    if lmouse_pressed {
                        let delta = cursor_curr - cursor_prev;

                        if shift_pressed {
                            // 4D rotation
                            if alt_pressed {
                                let rot_xy = rotations::get_simple_rotation_matrix(
                                    rotations::Plane::XY,
                                    delta.x,
                                );
                                let rot_zw = rotations::get_simple_rotation_matrix(
                                    rotations::Plane::ZW,
                                    delta.y,
                                );
                                four_rotation = rot_xy * rot_zw * four_rotation;
                            } else {
                                let rot_xw = rotations::get_simple_rotation_matrix(
                                    rotations::Plane::XW,
                                    delta.x,
                                );
                                let rot_yw = rotations::get_simple_rotation_matrix(
                                    rotations::Plane::YW,
                                    delta.y,
                                );
                                four_rotation = rot_xw * rot_yw * four_rotation;
                            }
                        } else {
                            // 3D rotation
                            let rot_xz = Matrix4::from_angle_y(cgmath::Rad(delta.x));
                            let rot_yz = Matrix4::from_angle_z(cgmath::Rad(delta.y));
                            three_rotation = rot_yz * rot_xz * three_rotation;
                        }
                    }
                }
                glutin::WindowEvent::MouseInput { state, button, .. } => match button {
                    glutin::MouseButton::Left => {
                        if let glutin::ElementState::Pressed = state {
                            cursor_pressed = cursor_curr;
                            lmouse_pressed = true;
                        } else {
                            lmouse_pressed = false;
                        }
                    }
                    glutin::MouseButton::Right => {
                        if let glutin::ElementState::Pressed = state {
                            rmouse_pressed = true;
                        } else {
                            rmouse_pressed = false;
                        }
                    }
                    _ => (),
                },
                glutin::WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(key) = input.virtual_keycode {
                        match input.state {
                            glutin::ElementState::Pressed => match key {
                                glutin::VirtualKeyCode::S => {
                                    let path = Path::new("frame.png");
                                    save_frame(path, WIDTH, HEIGHT);
                                }
                                glutin::VirtualKeyCode::O => {
                                    if draw_index > 0 {
                                        draw_index -= 1;
                                    }
                                }
                                glutin::VirtualKeyCode::P => {
                                    draw_index += 1;
                                    draw_index = draw_index.min(polytopes.len() - 1);
                                }
                                glutin::VirtualKeyCode::LShift => {
                                    shift_pressed = true;
                                }
                                glutin::VirtualKeyCode::LAlt => {
                                    alt_pressed = true;
                                }
                                _ => (),
                            },
                            glutin::ElementState::Released => match key {
                                glutin::VirtualKeyCode::LShift => {
                                    shift_pressed = false;
                                }
                                glutin::VirtualKeyCode::LAlt => {
                                    alt_pressed = false;
                                }
                                _ => (),
                            },
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

        three_rotation = Matrix4::from_angle_y(cgmath::Rad(milliseconds));

        program.uniform_1f("u_time", milliseconds);

        // Uniforms for 4D -> 3D projection.
        program.uniform_4f("u_four_from", &four_cam.from);
        program.uniform_matrix_4f("u_four_rotation", &four_rotation);
        program.uniform_matrix_4f("u_four_view", &four_cam.look_at);
        program.uniform_matrix_4f("u_four_projection", &four_cam.projection);

        // Uniforms for 3D -> 2D projection.
        program.uniform_matrix_4f("u_three_rotation", &three_rotation);
        program.uniform_matrix_4f("u_three_view", &three_view);
        program.uniform_matrix_4f("u_three_projection", &three_projection);

        clear();

        for tetra in tetrahedrons.iter() {
            program.uniform_4f("u_draw_color", &tetra.color);
            let tetra_slice = tetra.slice(&hyperplane);
            renderer.draw_tetrahedron_slice(&tetra_slice);
        }

        program.uniform_4f("u_draw_color", &Vector4::new(0.2, 0.5, 0.8, 1.0));
        polytopes[draw_index].draw();

        //hyperplane.displacement = (milliseconds * 0.5).sin() * 2.5;
        if rmouse_pressed {
            hyperplane.displacement = (cursor_curr.x * 2.0 - 1.0) * 2.5;
        }

        program.uniform_4f("u_draw_color", &Vector4::new(0.0, 1.0, 0.0, 0.25));
        for tetra in tetrahedrons.iter() {
            renderer.draw_tetrahedron(&tetra);
        }

        gl_window.swap_buffers().unwrap();
    }
}
