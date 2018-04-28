use cgmath::{self, Vector4};

pub fn from_hex(code: u32, alpha: f32) -> Vector4<f32> {
    let r = ((code >> 16) & 0xFF) as f32 / 255.0;
    let g = ((code >> 8) & 0xFF) as f32 / 255.0;
    let b = ((code) & 0xFF) as f32 / 255.0;
    Vector4::new(r, g, b, alpha)
}