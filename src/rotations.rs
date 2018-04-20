use cgmath::{self, Matrix4, Vector3, Vector4, InnerSpace};

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

/// Given a set of four vertices embedded in 4-dimensions, find a proper ordering
/// of `points[0]`, `points[1]`, `points[2]`, and `points[3]` such that the resulting
/// list of vertices can be drawn as two distinct triangles.
pub fn sort_quadrilateral(points: &Vec<Vector4<f32>>) -> Vec<Vector4<f32>> {
    assert_eq!(points.len(), 4);

    // First, project the 4D points to 3D.
    let align_with_x_axis = align();
    let projected = points.iter().map(|pt| {
        (align_with_x_axis * pt).truncate_n(0)
    }).collect::<Vec<_>>();

    assert_eq!(projected.len(), 4);

    // Now, we are safely working in 3-dimensions.
    let a = projected[0];
    let b = projected[1];
    let c = projected[2];

    let centroid = projected.iter().sum::<Vector3<f32>>() / projected.len() as f32;

    // Calculate the normal of this polygon by taking the cross product
    // between two of its edges.
    let ab = b - a;
    let bc = c - b;
    let polygon_normal = bc.cross(ab).normalize();

    let first_edge = (a - centroid).normalize();

    // Sort the new set of 3D points based on their signed angles.
    let mut indices = Vec::new();
    for pt in projected.iter().skip(1) {
        let edge = (pt - centroid).normalize();
        let angle = first_edge.dot(edge).max(-1.0).min(1.0);
        let mut signed_angle = angle.acos();

        if polygon_normal.dot(first_edge.cross(edge)) < 0.0 {
            signed_angle *= -1.0;
        }

        let index = indices.len() + 1;

        indices.push((index, signed_angle));
    }

    // Add the first point `a`.
    indices.push((0, 0.0));
    indices.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    // Now, return the original set of 4D points in the proper order.
    let points_sorted = indices.iter().map(|(index, _)| {
        points[*index]
    }).collect::<Vec<_>>();

    points_sorted
}

/// Construct a 4x4 matrix representing a series of plane rotations that cause
/// the vector <1, 1, 1, 1> to algin with the x-axis, <1, 0, 0, 0>.
///
/// Reference: `https://en.wikipedia.org/wiki/User:Tetracube/Coordinates_of_uniform_polytopes#Mapping_coordinates_back_to_n-space`
pub fn align() -> Matrix4<f32> {
    const DIMENSION: f32 = 4.0;

    Matrix4::from_cols(
        Vector4::new((1.0 / DIMENSION).sqrt(), -((DIMENSION - 1.0) / DIMENSION).sqrt(), 0.0, 0.0),
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
