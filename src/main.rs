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
use std::time::{Duration, SystemTime};

use gl::types::*;
use glutin::GlContext;
use cgmath::{InnerSpace, Matrix2, Matrix3, Matrix4, Perspective, Point2, Point3, Transform, SquareMatrix,
             Vector3, Vector4, Zero};

fn clear() {
    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}

struct FourCamera {
    from: Vector4<f32>,
    to: Vector4<f32>,
    up: Vector4<f32>,
    over: Vector4<f32>,
    look_at: Matrix4<f32>
}

impl FourCamera {
    fn new(
        from: Vector4<f32>,
        to: Vector4<f32>,
        up: Vector4<f32>,
        over: Vector4<f32>,
    ) -> FourCamera {
        let mut cam = FourCamera { from, to, up, over, look_at: Matrix4::identity() };
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

fn project(points: &Vec<Vector4<f32>>, cam: &FourCamera, t: f32) -> Vec<Vector4<f32>> {
    let mut projected = Vec::new();
    let four_view = cam.look_at;

    // Rotation in the ZW plane
//    let rot = Matrix4::from_cols(
//        Vector4::new(1.0, 0.0, 0.0, 0.0),
//        Vector4::new(0.0, 1.0, 0.0, 0.0),
//        Vector4::new(0.0, 0.0, t.cos(), t.sin()),
//        Vector4::new(0.0, 0.0, -t.sin(), t.cos())
//    );

    // Rotation in the YZ plane
    let rot = Matrix4::from_cols(
        Vector4::new(1.0, 0.0, 0.0, 0.0),
        Vector4::new(0.0, t.cos(), -t.sin(), 0.0),
        Vector4::new(0.0, t.sin(), t.cos(), 0.0),
        Vector4::new(0.0, 0.0, 0.0, 1.0)
    );

    for pt in points.iter() {
        // perspective projection to 3D
        if true {
            let t = 1.0f32 / (std::f32::consts::PI/4.0 * 0.5f32).tan();
            let mut v = *pt;// rot * (*pt);
            v -= cam.from;
            let s = t / v.dot(four_view.w);

            projected.push(Vector4::new(
                s * v.dot(four_view.x),
                s * v.dot(four_view.y),
                 s * v.dot(four_view.z),
                1.0
            ));
        }

        // parallel projection to 3D
        //*pt /= pt.w;
    }
    projected
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

    let mut points = vec![
        // all positive
        Vector4::new(1.0, 1.0, 1.0, 1.0),
        // one negative
        Vector4::new(-1.0, 1.0, 1.0, 1.0),
        Vector4::new(1.0, -1.0, 1.0, 1.0),
        Vector4::new(1.0, 1.0, -1.0, 1.0),
        Vector4::new(1.0, 1.0, 1.0, -1.0),
        // two negatives
        Vector4::new(-1.0, -1.0, 1.0, 1.0),
        Vector4::new(1.0, -1.0, -1.0, 1.0),
        Vector4::new(1.0, 1.0, -1.0, -1.0),
        Vector4::new(-1.0, 1.0, -1.0, 1.0),
        Vector4::new(-1.0, 1.0, 1.0, -1.0),
        Vector4::new(1.0, -1.0, 1.0, -1.0),
        // three negatives
        Vector4::new(-1.0, -1.0, -1.0, 1.0),
        Vector4::new(1.0, -1.0, -1.0, -1.0),
        Vector4::new(-1.0, 1.0, -1.0, -1.0),
        Vector4::new(-1.0, -1.0, 1.0, -1.0),
        // all negative
        Vector4::new(-1.0, -1.0, -1.0, -1.0),
    ];

//    let mut four_cam = FourCamera::new(
//        Vector4::new(4.0, 0.0, 0.0, 0.0),
//        Vector4::zero(),
//        Vector4::new(0.0, 1.0, 0.0, 0.0),
//        Vector4::new(0.0, 0.0, 1.0, 0.0),
//    );
    let mut four_cam = FourCamera::new(
        Vector4::new(2.83, 2.83, 0.01, 0.0),
        Vector4::zero(),
        Vector4::new(-0.71, 0.71, 0.0, 0.0),
        Vector4::new(0.0, 0.0, 1.0, 0.02),
    );

    let three_view = Matrix4::look_at(
        Point3::new(3.0, 0.99, 1.82),
        Point3::new(0.0, 0.0, 0.0),
        Vector3::unit_y(),
    );
    let three_projection = cgmath::perspective(cgmath::Rad(std::f32::consts::FRAC_PI_2), 1.0, 0.1, 1000.0);

    static VS_SRC: &'static str = "
    #version 430

    #define pi 3.1415926535897932384626433832795

    uniform vec4 u_four_from;
    uniform mat4 u_four_view;

    uniform mat4 u_three_view;
    uniform mat4 u_three_projection;

    layout(location = 0) in vec4 position;

    out float z_depth;

    float linear_depth(float z, float n, float f)
    {
        float d = 2.0 * z - 1.0;
        d = 2.0 * n * f / (f + n - d * (f - n));
        return d;
    }

    void main() {
        vec4 pos = position;

        // project 4D -> 3D
        if (false)
        {
            float t = 1.0 / tan(pi * 0.25 * 0.5);
            vec4 v = pos - u_four_from;

            float s = t / dot(v, u_four_view[3]);
            pos.x = s * dot(v, u_four_view[0]);
            pos.y = s * dot(v, u_four_view[1]);
            pos.z = s * dot(v, u_four_view[2]);
            pos.w = 1.0;
        }

        // project 3D -> 2D
        gl_Position = u_three_projection * u_three_view * pos;
        gl_PointSize = 8.0;

        // pass data to fragment shader
        z_depth = linear_depth(pos.z, 0.1, 1000.0);
    }";

    static FS_SRC: &'static str = "
    #version 430
    in float z_depth;
    layout(location = 0) out vec4 o_color;
    void main() {
        o_color = vec4(vec3(z_depth + 0.1), 1.0);
    }";
    let program = Program::new(VS_SRC.to_string(), FS_SRC.to_string()).unwrap();
    //program.uniform_4f("u_four_from", &four_cam.from);
    //program.uniform_matrix_4f("u_four_view", &four_view);
    program.uniform_matrix_4f("u_three_view", &three_view);
    program.uniform_matrix_4f("u_three_projection", &three_projection);


    let mut vao = 0;
    let mut vbo = 0;
    unsafe {
        gl::Enable(gl::VERTEX_PROGRAM_POINT_SIZE);

        // Create the OpenGL handles.
        gl::CreateVertexArrays(1, &mut vao);
        let vbo_size = (points.len() * mem::size_of::<Vector4<f32>>()) as GLsizeiptr;
        let attribute = 0;
        let bindpoint = 0;
        gl::CreateBuffers(1, &mut vbo);
        gl::NamedBufferData(
            vbo,
            vbo_size,
            points.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW,
        );

        // Set up vertex attribute(s).
        let num_elements = 4;
        gl::EnableVertexArrayAttrib(vao, attribute);
        gl::VertexArrayAttribFormat(vao, attribute, num_elements, gl::FLOAT, gl::FALSE, 0);
        gl::VertexArrayAttribBinding(vao, attribute, bindpoint);

        // Link vertex buffers to vertex attributes, via bindpoints.
        let offset = 0;
        gl::VertexArrayVertexBuffer(
            vao,
            bindpoint,
            vbo,
            offset,
            mem::size_of::<Vector4<f32>>() as i32,
        );
    }

    let start = SystemTime::now();
    loop {
        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::Closed => (),
                _ => (),
            },
            _ => (),
        });

        clear();

        // Retrieve the number of milliseconds since application launch.
        let elapsed = start.elapsed().unwrap();
        let milliseconds = elapsed.as_secs() * 1000 + elapsed.subsec_nanos() as u64 / 1_000_000;
        let ms = (milliseconds as f32) / 1000.0;
        //four_cam.from.z = 4.0 * ms.sin();
        //four_cam.from.y = 4.0 * ms.cos();
        four_cam.build_look_at();

        // Project the points from 4D -> 3D.
        let projected = project(&points, &four_cam, ms);

        // Update GPU buffer.
        unsafe {
            let data_size = (projected.len() * mem::size_of::<Vector4<f32>>()) as GLsizeiptr;
            gl::NamedBufferSubData(vbo, 0, data_size, projected.as_ptr() as *const GLvoid);
        }

        program.bind();
        unsafe {
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::POINTS, 0, projected.len() as i32);
        }

        gl_window.swap_buffers().unwrap();
    }
}
