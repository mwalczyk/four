#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_must_use)]
#![allow(unused_assignments)]
#![allow(unreachable_code)]
#![allow(unreachable_patterns)]
extern crate cgmath;
extern crate gl;
extern crate glutin;
extern crate image;

// Module imports.
mod camera;
mod constants;
mod hyperplane;
mod interaction;
mod mesh;
mod polychora;
mod program;
mod rotations;
mod tetrahedron;
mod utilities;

// Struct and function imports.
use camera::{Camera, FourCamera, ThreeCamera};
use hyperplane::Hyperplane;
use interaction::InteractionState;
use mesh::Mesh;
use polychora::Polychoron;
use program::Program;

use std::path::Path;
use std::time::{Duration, SystemTime};

use cgmath::{
    Array, Matrix4, Perspective, Point2, Point3, Rotation, SquareMatrix, Transform, Vector3,
    Vector4, Zero,
};
use glutin::GlContext;

/// Clears the default OpenGL framebuffer (color and depth).
fn clear() {
    unsafe {
        gl::ClearColor(0.1, 0.05, 0.05, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}

/// Sets project specific draw state.
fn set_draw_state() {
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

        // Allow the vertex shader to set the size of point sprites.
        gl::Enable(gl::PROGRAM_POINT_SIZE);
    }
}

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_dimensions(constants::WIDTH, constants::HEIGHT)
        .with_title("four");
    let context = glutin::ContextBuilder::new().with_multisampling(8);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();
    unsafe { gl_window.make_current() }.unwrap();
    gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

    set_draw_state();

    // Set up the slicing hyperplane.
    let mut hyperplane = Hyperplane::new(Vector4::unit_w(), 0.1);

    // Load several polychora and compute their tetrahedral decompositions.
    let mut meshes = vec![
        Mesh::new(Polychoron::Cell8),
        Mesh::new(Polychoron::Cell16),
        Mesh::new(Polychoron::Cell24),
        Mesh::new(Polychoron::Cell120)
    ];

    // Set up the model matrices, in 3-space.
    let mut model_matrices = vec![
        Matrix4::from_translation(Vector3::unit_x() * -3.5),
        Matrix4::from_translation(Vector3::unit_x() * -1.0),
        Matrix4::from_translation(Vector3::unit_x() * 1.0),
        Matrix4::from_translation(Vector3::unit_x() * 3.5),
    ];

    // Set up the "model" matrix, in 4-space.
    let mut rotation_in_4d = Matrix4::identity();

    // Initialize the camera that will be used to perform the 4D -> 3D projection.
    let four_cam = FourCamera::new(
        Vector4::unit_x() * 1.25,
        Vector4::zero(),
        Vector4::unit_y(),
        Vector4::unit_z(),
    );

    // Initialize the camera that will be used to perform the 3D -> 2D projection.
    let mut three_cam = ThreeCamera::new(
        Point3::new(0.0, 0.5, 4.0),
        Point3::from_value(0.0),
        Vector3::unit_y(),
    );

    // Load the shader programs that we will use for rendering.
    let slice_program = Program::two_stage(
        utilities::load_file_as_string(Path::new("shaders/shader.vert")),
        utilities::load_file_as_string(Path::new("shaders/shader.frag")),
    )
    .unwrap();

    let projections_program = Program::two_stage(
        utilities::load_file_as_string(Path::new("shaders/projections.vert")),
        utilities::load_file_as_string(Path::new("shaders/projections.frag")),
    )
    .unwrap();

    // Set up objects for interaction state.
    let mut interaction = InteractionState::new();
    let mut mode = 0;

    // Set up timing information (can be used inside of the shaders to animate objects).
    let start = SystemTime::now();

    loop {
        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::Closed => (),
                glutin::WindowEvent::MouseMoved { position, .. } => {
                    // Store the normalized mouse position.
                    interaction.cursor_prev = interaction.cursor_curr;
                    interaction.cursor_curr.x = position.0 as f32 / constants::WIDTH as f32;
                    interaction.cursor_curr.y = position.1 as f32 / constants::HEIGHT as f32;

                    if interaction.lmouse_pressed {
                        let delta = interaction.get_mouse_delta() * constants::MOUSE_SENSITIVITY;

                        if interaction.shift_pressed {
                            let rot = true;

                            let rot_xw = rotations::get_simple_rotation_matrix(
                                rotations::Plane::XW,
                                delta.x,
                            );
                            let rot_yw = rotations::get_simple_rotation_matrix(
                                rotations::Plane::YW,
                                delta.y,
                            );
                            rotation_in_4d = rot_xw * rot_yw * rotation_in_4d;
                        } else if interaction.ctrl_pressed {
                            let rot_zw = rotations::get_simple_rotation_matrix(
                                rotations::Plane::ZW,
                                delta.x,
                            );
                            let rot_zx = rotations::get_simple_rotation_matrix(
                                rotations::Plane::ZX,
                                delta.y,
                            );
                            rotation_in_4d = rot_zw * rot_zx * rotation_in_4d;
                        } else {
                            let rot_xz = Matrix4::from_angle_y(cgmath::Rad(delta.x));
                            let rot_yz = Matrix4::from_angle_x(cgmath::Rad(delta.y));

                            for model in model_matrices.iter_mut() {
                                *model = rot_yz * *model;
                            }
                        }
                    }
                }
                glutin::WindowEvent::MouseInput { state, button, .. } => match button {
                    glutin::MouseButton::Left => {
                        if let glutin::ElementState::Pressed = state {
                            interaction.cursor_pressed = interaction.cursor_curr;
                            interaction.lmouse_pressed = true;
                        } else {
                            interaction.lmouse_pressed = false;
                        }
                    }
                    glutin::MouseButton::Right => {
                        if let glutin::ElementState::Pressed = state {
                            interaction.rmouse_pressed = true;
                        } else {
                            interaction.rmouse_pressed = false;
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
                                    utilities::save_frame(
                                        path,
                                        constants::WIDTH,
                                        constants::HEIGHT,
                                    );
                                }
                                glutin::VirtualKeyCode::LShift => {
                                    interaction.shift_pressed = true;
                                }
                                glutin::VirtualKeyCode::LControl => {
                                    interaction.ctrl_pressed = true;
                                }
                                glutin::VirtualKeyCode::T => {
                                    mode += 1;
                                    mode = mode % 3;
                                }
                                glutin::VirtualKeyCode::W => unsafe {
                                    gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
                                },
                                glutin::VirtualKeyCode::F => unsafe {
                                    gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
                                },
                                glutin::VirtualKeyCode::H => {
                                    rotation_in_4d = Matrix4::identity();
                                }
                                _ => (),
                            },
                            glutin::ElementState::Released => match key {
                                glutin::VirtualKeyCode::LShift => {
                                    interaction.shift_pressed = false;
                                }
                                glutin::VirtualKeyCode::LControl => {
                                    interaction.ctrl_pressed = false;
                                }
                                _ => (),
                            },
                        }
                    }
                }
                glutin::WindowEvent::MouseWheel { delta, .. } => {
                    if let glutin::MouseScrollDelta::LineDelta(_, line_y) = delta {
                        let mut current_from = three_cam.get_from();

                        if line_y == 1.0 {
                            current_from.x -= constants::ZOOM_INCREMENT;
                        } else {
                            current_from.x += constants::ZOOM_INCREMENT;
                        }

                        three_cam.set_from(&current_from);
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
        clear();

        // Uniforms for 4D -> 3D projection.
        projections_program.uniform_1f("u_time", milliseconds);
        projections_program.uniform_4f("u_four_from", &four_cam.from);
        projections_program.uniform_matrix_4f("u_four_model", &rotation_in_4d);
        projections_program.uniform_matrix_4f("u_four_view", &four_cam.look_at);
        projections_program.uniform_matrix_4f("u_four_projection", &four_cam.projection);

        // Uniforms for 3D -> 2D projection.
        projections_program.uniform_matrix_4f("u_three_view", &three_cam.get_look_at());
        projections_program.uniform_matrix_4f("u_three_projection", &three_cam.get_projection());

        // TODO: the shader below is redundant and should be consolidated with `projections_program`
        // Uniforms for 3D -> 2D projection.
        slice_program.uniform_1f("u_time", milliseconds);
        slice_program.uniform_matrix_4f("u_view", &three_cam.get_look_at());
        slice_program.uniform_matrix_4f("u_projection", &three_cam.get_projection());

        match mode {
            0 => {
                // (0) Draw the results of the slicing operations.
                for mesh in meshes.iter_mut() {
                    mesh.set_transform(&rotation_in_4d);
                    mesh.slice(&hyperplane);
                }

                slice_program.bind();

                for (i, mesh) in meshes.iter().enumerate() {
                    slice_program.uniform_matrix_4f("u_model", &model_matrices[i]);
                    mesh.draw_slice();
                }
            }
            1 => {
                projections_program.bind();

                // (1) Draw the wireframes of all of the tetrahedra that make up the polychora.
                for (i, mesh) in meshes.iter().enumerate() {
                    projections_program.uniform_matrix_4f("u_three_model", &model_matrices[i]);
                    mesh.draw_tetrahedra();
                }
            }
            2 => {
                projections_program.bind();

                // (2) Draw the skeletons (wireframes) of the polychora.
                for (i, mesh) in meshes.iter().enumerate() {
                    projections_program.uniform_matrix_4f("u_three_model", &model_matrices[i]);
                    mesh.draw_edges();
                }
            }
            _ => (),
        }

        // Pressing the right mouse button and moving left <-> right will translate the
        // slicing hyperplane away from the origin.
        if interaction.rmouse_pressed {
            hyperplane.displacement =
                (interaction.cursor_curr.x * 2.0 - 1.0) * constants::W_DEPTH_RANGE;

            // Prevent this from ever becoming zero.
            if hyperplane.displacement == 0.0 {
                hyperplane.displacement += constants::EPSILON;
            }
        }

        gl_window.swap_buffers().unwrap();
    }
}
