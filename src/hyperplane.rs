use cgmath::{self, InnerSpace, Vector4};

pub struct Hyperplane {
    pub normal: Vector4<f32>,
    pub displacement: f32,
}

impl Hyperplane {
    pub fn new(mut normal: Vector4<f32>, displacement: f32) -> Hyperplane {
        // Make sure `normal` is of unit length.
        normal = normal.normalize();

        Hyperplane {
            normal,
            displacement,
        }
    }

    pub fn side(&self, point: &Vector4<f32>) -> f32 {
        self.normal.dot(*point) + self.displacement
    }
}
