use cgmath::{self, InnerSpace, Matrix4, SquareMatrix, Vector4};
use std::f32;

use rotations::cross;

pub struct Camera {
    pub from: Vector4<f32>,
    pub to: Vector4<f32>,
    pub up: Vector4<f32>,
    pub over: Vector4<f32>,
    pub look_at: Matrix4<f32>,
    pub projection: Matrix4<f32>,
}

impl Camera {
    pub fn new(
        from: Vector4<f32>,
        to: Vector4<f32>,
        up: Vector4<f32>,
        over: Vector4<f32>,
    ) -> Camera {
        let mut cam = Camera {
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

    pub fn build_look_at(&mut self) {
        let wd = (self.to - self.from).normalize();
        let wa = cross(&self.up, &self.over, &wd).normalize();
        let wb = cross(&self.over, &wd, &wa).normalize();
        let wc = cross(&wd, &wa, &wb);

        self.look_at = Matrix4::from_cols(wa, wb, wc, wd);
    }

    pub fn build_projection(&mut self) {
        let t = 1.0 / (f32::consts::FRAC_PI_4 * 0.5).tan();

        self.projection = Matrix4::from_diagonal(Vector4::new(t, t, t, t));
    }
}
