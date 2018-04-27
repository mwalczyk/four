use cgmath::{self, InnerSpace, Matrix4, Vector4};

#[derive(Copy, Clone, Debug)]
pub struct Hyperplane {
    pub normal: Vector4<f32>,
    pub displacement: f32,
}

impl Hyperplane {
    pub fn new(mut normal: Vector4<f32>, displacement: f32) -> Hyperplane {
        // Make sure `normal` is of unit length.
        //normal = normal.normalize();

        Hyperplane {
            normal,
            displacement
        }
    }

    pub fn inside(&self, point: &Vector4<f32>) -> bool {
        let pct = self.normal.dot(*point) - 2.0;

        //pct.abs() <= 0.381

         (self.normal.dot(*point)).abs() <= 0.001
    }

    pub fn side(&self, point: &Vector4<f32>) -> f32 {
        self.normal.dot(*point) + self.displacement
    }

    pub fn get_inverse_rotation(&self) -> Matrix4<f32> {
        const DIMENSION: f32 = 4.0;

        Matrix4::from_cols(
            Vector4::new(
                (1.0 / DIMENSION).sqrt(),
                -((DIMENSION - 1.0) / DIMENSION).sqrt(),
                0.0,
                0.0,
            ),
            Vector4::new(
                (1.0 / DIMENSION).sqrt(),
                (1.0 / (DIMENSION * (DIMENSION - 1.0))).sqrt(),
                -((DIMENSION - 2.0) / (DIMENSION - 1.0)).sqrt(),
                0.0,
            ),
            Vector4::new(
                (1.0 / DIMENSION).sqrt(),
                (1.0 / (DIMENSION * (DIMENSION - 1.0))).sqrt(),
                (1.0 / ((DIMENSION - 1.0) * (DIMENSION - 2.0))).sqrt(),
                -((DIMENSION - 3.0) / (DIMENSION - 2.0)).sqrt(),
            ),
            Vector4::new(
                (1.0 / DIMENSION).sqrt(),
                (1.0 / (DIMENSION * (DIMENSION - 1.0))).sqrt(),
                (1.0 / ((DIMENSION - 1.0) * (DIMENSION - 2.0))).sqrt(),
                (1.0 / ((DIMENSION - 2.0) * (DIMENSION - 3.0))).sqrt(),
            ),
        )
    }
}
