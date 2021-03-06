use cgmath::{self, InnerSpace, Matrix4, Vector3, Vector4, Zero};

use hyperplane::Hyperplane;
use utilities;

/// An enumeration representing a plane of rotation in 4D space.
pub enum Plane {
    XY,
    YZ,
    ZX,
    XW,
    YW,
    ZW,
}

/// Converts a set of hyperspherical coordinates `(r, ψ, φ, θ)` to Cartesian `(x, y, z, w)`
/// coordinates.
///
/// Reference: `http://mathworld.wolfram.com/Hypersphere.html`
pub fn hyperspherical_to_cartesian(r: f32, psi: f32, phi: f32, theta: f32) -> Vector4<f32> {
    let x = r * psi.sin() * phi.sin() * theta.cos();
    let y = r * psi.sin() * phi.sin() * theta.sin();
    let z = r * psi.sin() * phi.cos();
    let w = r * psi.cos();
    Vector4::new(x, y, z, w)
}

/// Returns `true` if `pt` is inside the hypersphere with radius `r` centered at the origin
/// and `false` otherwise.
pub fn inside_hypersphere(pt: &Vector4<f32>, r: f32) -> bool {
    pt.dot(*pt) <= (r * r)
}

/// Takes a 4D cross product between `u`, `v`, and `w`. The result is a vector in
/// 4-dimensions that is simultaneously orthogonal to `u`, `v`, and `w`.
///
/// Reference: `https://ef.gy/linear-algebra:normal-vectors-in-higher-dimensional-spaces`
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

/// 4-dimensional rotations are best thought about as rotations parallel to a plane.
/// For any of the six rotations below, only two coordinates change. In the future,
/// it might be interesting to explore the 4D equivalent of quaternions: rotors.
///
/// Reference: `https://math.stackexchange.com/questions/1402362/rotation-in-4d` and `http://hollasch.github.io/ray4/Four-Space_Visualization_of_4D_Objects.html#rotmats`
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
/// The only fixed point is the origin. These are also known as Clifford rotations.
/// Clifford rotations can be decomposed into two independent, simultaneous plane
/// rotations (each of which can have a different "rate" of rotation, i.e. `alpha`
/// and `beta`).
///
/// If `alpha` and `beta` are equal and non-zero, then the rotation is called an
/// isoclinic rotation.
///
/// This function accepts the first plane of rotation as an argument. The second
/// plane of rotation is determined by the first (the pair of 2-planes are orthogonal
/// to one another). The resulting matrix represents a rotation by `alpha` about the
/// first plane and a rotation of `beta` about the second plane.
///
/// Reference: `https://en.wikipedia.org/wiki/Plane_of_rotation#Double_rotations`
pub fn get_double_rotation_matrix(first_plane: Plane, alpha: f32, beta: f32) -> Matrix4<f32> {
    match first_plane {
        // α-XY, β-ZW
        Plane::XY => {
            get_simple_rotation_matrix(Plane::XY, alpha)
                * get_simple_rotation_matrix(Plane::ZW, beta)
        }

        // α-YZ, β-XW
        Plane::YZ => {
            get_simple_rotation_matrix(Plane::YZ, alpha)
                * get_simple_rotation_matrix(Plane::XW, beta)
        }

        // α-ZX, β-YW
        Plane::ZX => {
            get_simple_rotation_matrix(Plane::ZX, alpha)
                * get_simple_rotation_matrix(Plane::YW, beta)
        }

        // α-XW, β-YZ
        Plane::XW => {
            get_simple_rotation_matrix(Plane::XW, alpha)
                * get_simple_rotation_matrix(Plane::YZ, beta)
        }

        // α-YW, β-ZX
        Plane::YW => {
            get_simple_rotation_matrix(Plane::YW, alpha)
                * get_simple_rotation_matrix(Plane::ZX, beta)
        }

        // α-ZW, β-XY
        Plane::ZW => {
            get_simple_rotation_matrix(Plane::ZW, alpha)
                * get_simple_rotation_matrix(Plane::XY, beta)
        }
    }
}

/// See the notes above in `get_double_rotation_matrix(...)`. This function is
/// mostly here for completeness.
fn get_isoclinic_rotation_matrix(first_plane: Plane, alpha_beta: f32) -> Matrix4<f32> {
    get_double_rotation_matrix(first_plane, alpha_beta, alpha_beta)
}

/// Given a set of vertices embedded in 4-dimensions that lie inside `hyperplane`,
/// find a proper ordering of the points such that the resulting list of vertices can
/// be traversed in order to create a fan of distinct, non-overlapping triangles. Note
/// that for the purposes of this application, we don't care if the list ends up
/// in a "clockwise" or "counter-clockwise" order.
///
/// Reference: `https://math.stackexchange.com/questions/978642/how-to-sort-vertices-of-a-polygon-in-counter-clockwise-order`
pub fn sort_points_on_plane(
    points: &Vec<Vector4<f32>>,
    hyperplane: &Hyperplane,
) -> Vec<Vector4<f32>> {
    let largest_index = utilities::index_of_largest(&hyperplane.normal);

    // First, project the 4D points to 3D. We do this by dropping the coordinate
    // corresponding to the largest value of the hyperplane's normal vector.
    //
    // TODO: does this work all the time?
    let projected = points
        .iter()
        .map(|pt| pt.truncate_n(largest_index as isize))
        .collect::<Vec<_>>();

    // Now, we are working in 3-dimensions.
    let a = projected[0];
    let b = projected[1];
    let c = projected[2];
    let centroid = utilities::average(&projected, &Vector3::zero());

    // Calculate the normal of this polygon by taking the cross product
    // between two of its edges.
    let ab = b - a;
    let bc = c - b;
    let polygon_normal = bc.cross(ab).normalize();
    let first_edge = (a - centroid).normalize();

    // Sort the new set of 3D points based on their signed angles.
    let mut indices = Vec::new();
    for point in projected.iter().skip(1) {
        let edge = (point - centroid).normalize();
        let angle = utilities::saturate_between(first_edge.dot(edge), -1.0, 1.0);
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
    let points_sorted = indices
        .iter()
        .map(|(index, _)| points[*index])
        .collect::<Vec<_>>();

    points_sorted
}

/// Construct a 4x4 matrix representing a series of plane rotations that cause
/// the vector <1, 1, 1, 1> to align with the x-axis, <1, 0, 0, 0>. This is useful
/// for projecting points from 4D -> 3D, if we decide to slice corner-first (which
/// is currently not the case).
///
/// Reference: `https://en.wikipedia.org/wiki/User:Tetracube/Coordinates_of_uniform_polytopes#Mapping_coordinates_back_to_n-space`
pub fn align_corner_to_x_axis() -> Matrix4<f32> {
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
