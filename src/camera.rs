use cgmath::{self, InnerSpace, Matrix4, Point3, SquareMatrix, Vector3, Vector4};

use std::f32;

use rotations::cross;

pub trait Camera {
    fn get_look_at(&self) -> &Matrix4<f32>;
    fn get_projection(&self) -> &Matrix4<f32>;
    fn build_look_at(&mut self);
    fn build_projection(&mut self);
}

pub struct FourCamera {
    pub from: Vector4<f32>,
    pub to: Vector4<f32>,
    pub up: Vector4<f32>,
    pub over: Vector4<f32>,
    pub look_at: Matrix4<f32>,
    pub projection: Matrix4<f32>,
}

impl FourCamera {
    pub fn new(
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
            projection: Matrix4::identity(),
        };
        cam.build_look_at();
        cam.build_projection();

        cam
    }
}

impl Camera for FourCamera {
    fn get_look_at(&self) -> &Matrix4<f32> {
        &self.look_at
    }

    fn get_projection(&self) -> &Matrix4<f32> {
        &self.projection
    }

    fn build_look_at(&mut self) {
        let wd = (self.to - self.from).normalize();
        let wa = cross(&self.up, &self.over, &wd).normalize();
        let wb = cross(&self.over, &wd, &wa).normalize();
        let wc = cross(&wd, &wa, &wb);

        self.look_at = Matrix4::from_cols(wa, wb, wc, wd);
    }

    fn build_projection(&mut self) {
        let t = 1.0 / (f32::consts::FRAC_PI_4 * 0.5).tan();

        self.projection = Matrix4::from_diagonal(Vector4::new(t, t, t, t));
    }
}

pub struct ThreeCamera {
    from: Point3<f32>,
    to: Point3<f32>,
    up: Vector3<f32>,
    look_at: Matrix4<f32>,
    projection: Matrix4<f32>,
}

impl ThreeCamera {
    pub fn new(
        from: Point3<f32>,
        to: Point3<f32>,
        up: Vector3<f32>,
    ) -> ThreeCamera {
        let mut cam = ThreeCamera {
            from,
            to,
            up,
            look_at: Matrix4::identity(),
            projection: Matrix4::identity(),
        };
        cam.build_look_at();
        cam.build_projection();

        cam
    }

    pub fn get_from(&self) -> Point3<f32> {
        self.from
    }

    pub fn set_from(&mut self, from: &Point3<f32>) {
        self.from = *from;
        self.build_look_at();
    }
}

impl Camera for ThreeCamera {
    fn get_look_at(&self) -> &Matrix4<f32> {
        &self.look_at
    }

    fn get_projection(&self) -> &Matrix4<f32> {
        &self.projection
    }

    fn build_look_at(&mut self) {
        self.look_at = Matrix4::look_at(self.from, self.to, self.up);
    }

    fn build_projection(&mut self) {
        let fov = cgmath::Rad(std::f32::consts::FRAC_PI_2);
        self.projection = cgmath::perspective(fov, 1.0, 0.1, 1000.0);
    }
}