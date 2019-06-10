use cgmath::{self, InnerSpace, Vector4};

use constants;

/// A 4-dimensional hyperplane, specified in Hessian normal form:
///
///     n.dot(x) = -d
///
/// where `d` is the distance of the hyperplane from the origin. Here, the sign
/// of `d` determines the side of the plane on which the origin is located. If
/// `d` is positive, it is in the half-space determined by the direction of `n`.
/// Otherwise, it is in the other half-space.
///
/// Reference: `http://mathworld.wolfram.com/HessianNormalForm.html`
#[derive(Copy, Clone, Debug)]
pub struct Hyperplane {
    pub normal: Vector4<f32>,
    pub displacement: f32,
}

impl Hyperplane {
    pub fn new(mut normal: Vector4<f32>, displacement: f32) -> Hyperplane {
        normal = normal.normalize();

        Hyperplane {
            normal,
            displacement,
        }
    }

    pub fn get_normal(&self) -> Vector4<f32> {
        self.normal
    }

    pub fn get_displacement(&self) -> f32 {
        self.displacement
    }

    /// Returns `true` if `point` is "inside" the hyperplane (within some epsilon) and
    /// `false` otherwise.
    pub fn inside(&self, point: &Vector4<f32>) -> bool {
        self.signed_distance(point).abs() <= constants::EPSILON
    }

    /// Returns the signed distance (in 4-space) of `point` to the hyperplane.
    ///
    /// Reference: `http://mathworld.wolfram.com/Point-PlaneDistance.html`
    pub fn signed_distance(&self, point: &Vector4<f32>) -> f32 {
        self.normal.dot(*point) + self.displacement
    }
}
