extern crate cgmath;
extern crate gl;
extern crate glutin;
extern crate image;

mod camera;
mod constants;
mod hyperplane;
mod interaction;
mod mesh;
mod polychora;
mod program;
mod renderer;
mod rotations;
mod tetrahedron;
mod utilities;

use camera::Camera;
use hyperplane::Hyperplane;
use interaction::InteractionState;
use mesh::Mesh;
use polychora::Polychoron;
use program::Program;
use renderer::Renderer;
use tetrahedron::Tetrahedron;

use std::path::Path;
use std::time::{Duration, SystemTime};

use cgmath::{Array, InnerSpace, Matrix2, Matrix3, Matrix4, Perspective, Point2, Point3, Rotation,
             SquareMatrix, Transform, Vector2, Vector3, Vector4, Zero};
use glutin::GlContext;

fn clear() {
    unsafe {
        gl::ClearColor(0.1, 0.05, 0.05, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}

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

    // Load the 120-cell and compute its tetrahedral decomposition.
    let mut mesh = Mesh::new(Polychoron::Cell120);
    let mut tetrahedrons = mesh.tetrahedralize();
    println!(
        "Mesh tetrahedralization resulted in {} tetrahedrons.",
        tetrahedrons.len()
    );

    // Set up the 4D camera - we don't really use this anymore, since we are performing an
    // orthographic projection to go from 4D -> 3D (see the shader).
    let four_cam = Camera::new(
        Vector4::unit_x() * 3.0,
        Vector4::zero(),
        Vector4::unit_y(),
        Vector4::unit_z(),
    );
    let mut four_rotation = Matrix4::identity();

    // Set up the 3D camera.
    let mut three_rotation = Matrix4::identity();
    let three_view = Matrix4::look_at(
        Point3::new(6.0, 0.0, 0.0),
        Point3::from_value(0.0),
        Vector3::unit_y(),
    );
    let three_projection =
        cgmath::perspective(cgmath::Rad(std::f32::consts::FRAC_PI_2), 1.0, 0.1, 1000.0);

    // Load the shader program that we will use for rendering.
    let program = utilities::load_shaders(
        Path::new("shaders/shader.vert"),
        Path::new("shaders/shader.frag"),
    );
    program.bind();

    let renderer = Renderer::new();

    let mut interaction = InteractionState::new();
    let mut show_tetrahedrons = false;
    let mut reveal_cells = mesh.def.cells;

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
                            let rot_xw = rotations::get_simple_rotation_matrix(
                                rotations::Plane::XW,
                                delta.x,
                            );
                            let rot_yw = rotations::get_simple_rotation_matrix(
                                rotations::Plane::YW,
                                delta.y,
                            );
                            four_rotation = rot_xw * rot_yw * four_rotation;
                        } else if interaction.ctrl_pressed {
                            let rot_zw = rotations::get_simple_rotation_matrix(
                                rotations::Plane::ZW,
                                delta.x,
                            );
                            let rot_zx = rotations::get_simple_rotation_matrix(
                                rotations::Plane::ZX,
                                delta.y,
                            );
                            four_rotation = rot_zw * rot_zx * four_rotation;
                        } else {
                            let rot_xz = Matrix4::from_angle_y(cgmath::Rad(delta.x));
                            let rot_yz = Matrix4::from_angle_z(cgmath::Rad(delta.y));
                            three_rotation = rot_yz * rot_xz * three_rotation;
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
                                    show_tetrahedrons = !show_tetrahedrons;
                                }
                                glutin::VirtualKeyCode::W => unsafe {
                                    gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
                                },
                                glutin::VirtualKeyCode::F => unsafe {
                                    gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
                                },
                                glutin::VirtualKeyCode::LBracket => {
                                    if reveal_cells > 0 {
                                        reveal_cells -= 1;
                                    }
                                }
                                glutin::VirtualKeyCode::RBracket => {
                                    if reveal_cells < mesh.def.cells {
                                        reveal_cells += 1;
                                    }
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
                _ => (),
            },
            _ => (),
        });

        // Retrieve the number of milliseconds since application launch.
        let elapsed = start.elapsed().unwrap();
        let seconds = elapsed.as_secs() * 1000 + elapsed.subsec_nanos() as u64 / 1_000_000;
        let milliseconds = (seconds as f32) / 1000.0;
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

        for tetra in tetrahedrons.iter_mut() {
            if tetra.cell < reveal_cells {
                program.uniform_4f("u_draw_color", &tetra.color);
                program.uniform_4f("u_cell_centroid", &tetra.cell_centroid);

                // First, set this tetrahedron's transform matrix
                tetra.set_transform(&four_rotation);

                // Then, render the slice
                renderer.draw_tetrahedron_slice(&tetra.slice(&hyperplane));
            }
        }

        // Draw the full polytope - for now, we leave this disabled.
        let draw_polytope = false;
        if draw_polytope {
            program.uniform_4f("u_draw_color", &Vector4::new(0.2, 0.5, 0.8, 1.0));
            mesh.draw();
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

        // Finally, draw the wireframe of all tetrahedrons that make up this 4D mesh.
        if show_tetrahedrons {
            program.uniform_4f("u_draw_color", &Vector4::new(0.0, 1.0, 0.0, 0.25));
            for tetra in tetrahedrons.iter() {
                renderer.draw_tetrahedron(&tetra);
            }
        }

        gl_window.swap_buffers().unwrap();
    }
}
