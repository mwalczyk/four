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
mod constants;
mod hyperplane;
mod polytope;
mod program;
mod renderer;
mod rotations;
mod tetrahedron;
mod utilities;

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

use cgmath::{Array, InnerSpace, Matrix2, Matrix3, Matrix4, Perspective, Point2, Point3, Rotation,
             SquareMatrix, Transform, Vector2, Vector3, Vector4, Zero};
use glutin::GlContext;
use image::{GenericImage, ImageBuffer};

fn clear() {
    unsafe {
        gl::ClearColor(0.1, 0.05, 0.05, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
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

        gl::Enable(gl::PROGRAM_POINT_SIZE);
    }

    // Set up the 4D shape(s).
    let mut hyperplane = Hyperplane::new(Vector4::unit_w(), 0.1);
    let mut polytopes = load_shapes();
    let mut tetrahedrons = polytopes[0].tetrahedralize();

    println!(
        "Mesh tetrahedralization resulted in {} tetrahedrons",
        tetrahedrons.len()
    );

    // TODO: this camera isn't really being used right now...
    let four_cam = Camera::new(
        Vector4::unit_x() * 3.0,
        Vector4::zero(),
        Vector4::unit_y(),
        Vector4::unit_z(),
    );
    let mut four_rotation = Matrix4::identity();

    let mut three_rotation = Matrix4::identity();
    let three_view = Matrix4::look_at(
        Point3::new(3.5, 0.0, 0.0),
        Point3::from_value(0.0),
        Vector3::unit_y(),
    );
    let three_projection =
        cgmath::perspective(cgmath::Rad(std::f32::consts::FRAC_PI_2), 1.0, 0.1, 1000.0);

    let program = load_shaders(
        Path::new("shaders/shader.vert"),
        Path::new("shaders/shader.frag"),
    );
    program.bind();

    let renderer = Renderer::new();
    let start = SystemTime::now();

    let mut cursor_prev = Vector2::zero();
    let mut cursor_curr = Vector2::zero();
    let mut cursor_pressed = Vector2::zero();
    let mut lmouse_pressed = false;
    let mut rmouse_pressed = false;
    let mut shift_pressed = false;
    let mut ctrl_pressed = false;
    let mut draw_index = 0;

    // Other controls.
    let mut show_tetrahedrons = true;
    let mut reveal_cells = 120;

    loop {
        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::Closed => (),
                glutin::WindowEvent::MouseMoved { position, .. } => {
                    cursor_prev = cursor_curr;
                    cursor_curr.x = position.0 as f32 / WIDTH as f32;
                    cursor_curr.y = position.1 as f32 / HEIGHT as f32;
                    if lmouse_pressed {
                        let delta = (cursor_curr - cursor_prev) * constants::MOUSE_SENSITIVITY;

                        if shift_pressed {
                            // 4D rotations (1)
                            let rot_xw = rotations::get_simple_rotation_matrix(
                                rotations::Plane::XW,
                                delta.x,
                            );
                            let rot_yw = rotations::get_simple_rotation_matrix(
                                rotations::Plane::YW,
                                delta.y,
                            );
                            four_rotation = rot_xw * rot_yw * four_rotation;
                        } else if ctrl_pressed {
                            // 4D rotations (2)
                            let rot_xy = rotations::get_simple_rotation_matrix(
                                rotations::Plane::XY,
                                delta.x,
                            );
                            let rot_zx = rotations::get_simple_rotation_matrix(
                                rotations::Plane::ZX,
                                delta.y,
                            );
                            four_rotation = rot_xy * rot_zx * four_rotation;
                        } else {
                            // 3D rotations
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
                                    //save_frame(path, WIDTH, HEIGHT);
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
                                glutin::VirtualKeyCode::LControl => {
                                    ctrl_pressed = true;
                                }
                                glutin::VirtualKeyCode::T => {
                                    show_tetrahedrons = !show_tetrahedrons;
                                }
                                glutin::VirtualKeyCode::LBracket => {
                                    if reveal_cells > 0 {
                                        reveal_cells -= 1;
                                    }
                                }
                                glutin::VirtualKeyCode::RBracket => {
                                    if reveal_cells < 120 {
                                        reveal_cells += 1;
                                    }
                                }
                                _ => (),
                            },
                            glutin::ElementState::Released => match key {
                                glutin::VirtualKeyCode::LShift => {
                                    shift_pressed = false;
                                }
                                glutin::VirtualKeyCode::LControl => {
                                    ctrl_pressed = false;
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
        program.uniform_1f("u_time", milliseconds);

        // Automatically rotate around the y-axis in 3-dimensions
        //three_rotation = Matrix4::from_angle_y(cgmath::Rad(milliseconds));

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

        //unsafe { gl::PolygonMode( gl::FRONT_AND_BACK, gl::LINE ); }

        for tetra in tetrahedrons.iter_mut() {

            if tetra.cell < reveal_cells {
                program.uniform_4f("u_draw_color", &tetra.color);

                // First, set this tetrahedron's transform matrix
                tetra.set_transform(&four_rotation);

                // Then, render the slice
                renderer.draw_tetrahedron_slice(&tetra.slice(&hyperplane));
            }
        }

        // Draw the full polytope
        //program.uniform_4f("u_draw_color", &Vector4::new(0.2, 0.5, 0.8, 1.0));
        //polytopes[draw_index].draw();

        // Pressing the right mouse button and moving left <-> right will translate the
        // slicing hyperplane away from the origin
        if rmouse_pressed {
            hyperplane.displacement = (cursor_curr.x * 2.0 - 1.0) * 4.5;

            // Prevent this from ever becoming zero
            if hyperplane.displacement == 0.0 {
                hyperplane.displacement += constants::EPSILON;
            }
        }

        // Finally, draw the wireframe of all tetrahedrons that make up this 4D mesh
        if show_tetrahedrons {
            program.uniform_4f("u_draw_color", &Vector4::new(0.0, 1.0, 0.0, 0.25));
            for tetra in tetrahedrons.iter() {
                renderer.draw_tetrahedron(&tetra);
            }
        }

        gl_window.swap_buffers().unwrap();
    }
}
