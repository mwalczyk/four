use cgmath::{self, ElementWise, Vector3, Vector4};

pub fn from_hex(code: u32, alpha: f32) -> Vector4<f32> {
    let r = ((code >> 16) & 0xFF) as f32 / 255.0;
    let g = ((code >> 8) & 0xFF) as f32 / 255.0;
    let b = ((code) & 0xFF) as f32 / 255.0;
    Vector4::new(r, g, b, alpha)
}

pub fn palette(
    t: f32,
    a: &Vector3<f32>,
    b: &Vector3<f32>,
    c: &Vector3<f32>,
    d: &Vector3<f32>,
) -> Vector3<f32> {
    use std::f32;

    // TODO: there should be a way to iterate over the `Vector4<f32>` and do this...
    let mut temp = (c * t + d) * 2.0 * f32::consts::PI;
    temp.x = temp.x.cos();
    temp.y = temp.y.cos();
    temp.z = temp.z.cos();

    a + b.mul_element_wise(temp)
}
