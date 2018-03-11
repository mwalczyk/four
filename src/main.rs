#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unreachable_code)]
extern crate cgmath;
extern crate gl;
extern crate glutin;

mod program;

use glutin::GlContext;
use cgmath::{Point2, Point3, Vector3, Vector4, Matrix2, Matrix3, Matrix4, InnerSpace, Zero, Perspective};

/// Takes a 4D cross product between `u`, `v`, and `w`.
fn cross4(u: Vector4<f32>, v: Vector4<f32>, w: Vector4<f32>) -> Vector4<f32> {
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

    let from = Point3::new(0.0, 0.0, 3.0);
    let to = Point3::new(0.0, 0.0, 0.0);
    let up = Vector3::unit_y();
    let lookat = lookat3(from, to, up);

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

    for pt in points.iter_mut() {
        // transform from world space to eye space
        pt.x -= from.x;
        pt.y -= from.y;
        pt.z -= from.z;
        pt = lookat * pt;
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
        gl_window.swap_buffers().unwrap();
    }
}
