#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unreachable_code)]
extern crate cgmath;
extern crate gl;
extern crate glutin;

mod program;

use program::Program;

use std::mem;
use std::ptr;

use gl::types::*;
use glutin::GlContext;
use cgmath::{Point2, Point3, Vector3, Vector4, Matrix2, Matrix3, Matrix4, InnerSpace, Zero, Perspective, Transform};

/// Takes a 4D cross product between `u`, `v`, and `w`.
fn cross4(u: &Vector4<f32>, v: &Vector4<f32>, w: &Vector4<f32>) -> Vector4<f32> {
    let a = (v[0] * w[1]) - (v[1] * w[0]);
    let b = (v[0] * w[2]) - (v[2] * w[0]);
    let c = (v[0] * w[3]) - (v[3] * w[0]);
    let d = (v[1] * w[2]) - (v[2] * w[1]);
    let e = (v[1] * w[3]) - (v[3] * w[1]);
    let f = (v[2] * w[3]) - (v[3] * w[2]);

    let result = Vector4::new((u[1] * f) - (u[2] * e) + (u[3] * d),
                             -(u[0] * f) + (u[2] * c) - (u[3] * b),
                              (u[0] * e) - (u[1] * c) + (u[3] * a),
                             -(u[0] * d) + (u[1] * b) - (u[2] * a));
    result
}

fn lookat4(from: Vector4<f32>, to: Vector4<f32>, up: Vector4<f32>, over: Vector4<f32>) -> Matrix4<f32> {
    let wd = (to - from).normalize();
    let wa = cross4(&up, &over, &wd).normalize();
    let wb = cross4(&over, &wd, &wa).normalize();
    let wc = cross4(&wd, &wa, &wb);

    Matrix4::from_cols(wa, wb, wc, wd)
}

fn lookat3(from: Point3<f32>, to: Point3<f32>, up: Vector3<f32>) -> Matrix3<f32> {
    let vc = (to - from).normalize();
    let va = vc.cross(up).normalize();
    let vb = va.cross(vc);

    Matrix3::from_cols(va, vb, vc)
}

fn clear() {
    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_dimensions(600, 600)
        .with_title("four");
    let context = glutin::ContextBuilder::new().with_multisampling(4);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();
    unsafe { gl_window.make_current() }.unwrap();
    gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

    // 4D -> 3D projection
    // ...

    // 3D -> 2D projection
    let from3 = Point3::new(0.0, 0.0, 3.0);
    let to3 = Point3::new(0.0, 0.0, 0.0);
    let up3 = Vector3::unit_y();
    let lookat3 = lookat3(from3, to3, up3);

    let mut points = vec![
        // base
        Vector3::new(-1.0, -1.0, -1.0),
        Vector3::new(-1.0, -1.0, 1.0),
        Vector3::new(1.0, -1.0, -1.0),
        Vector3::new(1.0, -1.0, 1.0),

        // top
        Vector3::new(-1.0, 1.0, -1.0),
        Vector3::new(-1.0, 1.0, 1.0),
        Vector3::new(1.0, 1.0, -1.0),
        Vector3::new(1.0, 1.0, 1.0),
    ];

    let fov = 45.0;
    let cx = 0.0;
    let cy = 0.0;
    let lx = 600.0;
    let ly = 600.0;
    let t = 1.0f32 / (fov * 0.5f32).tan();

    for pt in points.iter_mut() {
        // Transform from world space to eye space.
        pt.x -= from3.x;
        pt.y -= from3.y;
        pt.z -= from3.z;
        *pt = lookat3 * (*pt);

        // To keep things simple, we simply divide by `z`
        // in the shader program below. A more flexible
        // solution would incorporate `cx`, `cy`, etc.
        // ...
    }

    static VS_SRC: &'static str = "
    #version 430
    layout(location = 0) in vec3 position;
    void main() {
        gl_PointSize = 4.0;
        gl_Position = vec4(position.xy, 0.0, position.z);
    }";

    static FS_SRC: &'static str = "
    #version 430
    layout(location = 0) out vec4 o_color;
    void main() {
        o_color = vec4(1.0);
    }";
    let program = Program::new(VS_SRC.to_string(), FS_SRC.to_string()).unwrap();

    let mut vao = 0;
    let mut vbo = 0;
    unsafe  {
        gl::Enable(gl::VERTEX_PROGRAM_POINT_SIZE);

        // Create the OpenGL handles.
        gl::CreateVertexArrays(1, &mut vao);
        let vbo_size = (points.len() * mem::size_of::<Vector3<f32>>()) as GLsizeiptr;
        let attribute = 0;
        let bindpoint = 0;
        gl::CreateBuffers(1, &mut vbo);
        gl::NamedBufferData(vbo, vbo_size, points.as_ptr() as *const GLvoid, gl::STATIC_DRAW);

        // Set up vertex attribute(s).
        let num_elements = 3;
        gl::EnableVertexArrayAttrib(vao, attribute);
        gl::VertexArrayAttribFormat(vao, attribute, num_elements, gl::FLOAT, gl::FALSE, 0);
        gl::VertexArrayAttribBinding(vao, attribute, bindpoint);

        // Link vertex buffers to vertex attributes, via bindpoints.
        let offset = 0;
        gl::VertexArrayVertexBuffer(vao, bindpoint, vbo, offset, mem::size_of::<Vector3<f32>>() as i32);
    }

    loop {
        events_loop.poll_events(|event| {
            match event {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::Closed => (),
                    _ => ()
                }
                _ => ()
            }
        });

        clear();

        program.bind();
        unsafe {
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::POINTS, 0, points.len() as i32);
        }

        gl_window.swap_buffers().unwrap();
    }
}
