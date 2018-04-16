use cgmath::{self, Matrix4, Vector4};

pub enum Plane {
    XY,
    YZ,
    ZX,
    XW,
    YW,
    ZW,
}

/// Takes a 4D cross product between `u`, `v`, and `w`.
pub fn cross(u: &Vector4<f32>, v: &Vector4<f32>, w: &Vector4<f32>) -> Vector4<f32> {
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

/// The 4D equivalent of a quaternion is known as a rotor.
///
/// Reference: `https://math.stackexchange.com/questions/1402362/rotation-in-4d`
pub fn get_simple_rotation_matrix(plane: Plane, angle: f32) -> Matrix4<f32> {
    let c = angle.cos();
    let s = angle.sin();

    match plane {
        Plane::XY => Matrix4::from_cols(
            Vector4::new(c, -s, 0.0, 0.0),
            Vector4::new(s, c, 0.0, 0.0),
            Vector4::new(0.0, 0.0, 1.0, 0.0),
            Vector4::new(0.0, 0.0, 0.0, 1.0),
        ),
        Plane::YZ => Matrix4::from_cols(
            Vector4::new(1.0, 0.0, 0.0, 0.0),
            Vector4::new(0.0, c, -s, 0.0),
            Vector4::new(0.0, s, c, 0.0),
            Vector4::new(0.0, 0.0, 0.0, 1.0),
        ),
        Plane::ZX => Matrix4::from_cols(
            Vector4::new(c, 0.0, s, 0.0),
            Vector4::new(0.0, 1.0, 0.0, 0.0),
            Vector4::new(-s, 0.0, c, 0.0),
            Vector4::new(0.0, 0.0, 0.0, 1.0),
        ),
        Plane::XW => Matrix4::from_cols(
            Vector4::new(c, 0.0, 0.0, -s),
            Vector4::new(0.0, 1.0, 0.0, 0.0),
            Vector4::new(0.0, 0.0, 1.0, 0.0),
            Vector4::new(s, 0.0, 0.0, c),
        ),
        Plane::YW => Matrix4::from_cols(
            Vector4::new(1.0, 0.0, 0.0, 0.0),
            Vector4::new(0.0, c, 0.0, s),
            Vector4::new(0.0, 0.0, 1.0, 0.0),
            Vector4::new(0.0, -s, 0.0, c),
        ),
        Plane::ZW => Matrix4::from_cols(
            Vector4::new(1.0, 0.0, 0.0, 0.0),
            Vector4::new(0.0, 1.0, 0.0, 0.0),
            Vector4::new(0.0, 0.0, c, s),
            Vector4::new(0.0, 0.0, -s, c),
        ),
    }
}

/// Returns a "double rotation" matrix, which represents two planes of rotation.
/// The only fixed point is the origin. If `alpha` and `beta` are equal and non-zero,
/// then the rotation is called an isoclinic rotation.
///
/// Reference: `https://en.wikipedia.org/wiki/Plane_of_rotation#Double_rotations`
pub fn get_double_rotation_matrix(alpha: f32, beta: f32) -> Matrix4<f32> {
    let ca = alpha.cos();
    let sa = alpha.sin();
    let cb = beta.cos();
    let sb = beta.sin();

    Matrix4::from_cols(
        Vector4::new(ca, sa, 0.0, 0.0),
        Vector4::new(-sa, ca, 0.0, 0.0),
        Vector4::new(0.0, 0.0, cb, sb),
        Vector4::new(0.0, 0.0, -sb, cb),
    )
}

/// Construct a 4x4 matrix representing a series of plane rotations that cause
/// the vector <1, 1, 1, 1> to algin with the x-axis, <1, 0, 0, 0>.
///
/// Reference: `https://en.wikipedia.org/wiki/User:Tetracube/Coordinates_of_uniform_polytopes#Mapping_coordinates_back_to_n-space`
pub fn align() -> Matrix4<f32> {
    let n = 4.0f32;

    Matrix4::from_cols(
        Vector4::new((1.0 / n).sqrt(), -((n - 1.0) / n).sqrt(), 0.0, 0.0),
        Vector4::new(
            (1.0 / n).sqrt(),
            (1.0 / (n * (n - 1.0))).sqrt(),
            -((n - 2.0) / (n - 1.0)).sqrt(),
            0.0,
        ),
        Vector4::new(
            (1.0 / n).sqrt(),
            (1.0 / (n * (n - 1.0))).sqrt(),
            (1.0 / ((n - 1.0) * (n - 2.0))).sqrt(),
            -((n - 3.0) / (n - 2.0)).sqrt(),
        ),
        Vector4::new(
            (1.0 / n).sqrt(),
            (1.0 / (n * (n - 1.0))).sqrt(),
            (1.0 / ((n - 1.0) * (n - 2.0))).sqrt(),
            (1.0 / ((n - 2.0) * (n - 3.0))).sqrt(),
        ),
    )
}
