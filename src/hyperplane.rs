use cgmath::{self, InnerSpace, Matrix4, Vector4};

use constants;

#[derive(Copy, Clone, Debug)]
pub struct Hyperplane {
    pub normal: Vector4<f32>,
    pub displacement: f32,
}

impl Hyperplane {
    pub fn new(normal: Vector4<f32>, displacement: f32) -> Hyperplane {
        // TODO: for now, we don't need to do this?
        //normal = normal.normalize();

        Hyperplane {
            normal,
            displacement,
        }
    }

    pub fn inside(&self, point: &Vector4<f32>) -> bool {
        self.side(point).abs() <= constants::EPSILON
    }

    pub fn side(&self, point: &Vector4<f32>) -> f32 {
        self.normal.dot(*point) + self.displacement
    }
}
