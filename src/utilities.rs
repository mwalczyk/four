use std::fs::File;
use std::io::Read;
use std::iter;
use std::os::raw::c_void;
use std::path::Path;

use cgmath::{self, ElementWise, Vector3, Vector4};
use gl;
use gl::types::*;
use image::{self, GenericImage, ImageBuffer};

use program::Program;

/// Creates an RGBA color (represented as a vector) from a hex code and alpha.
/// For example, `from_hex(0xffffff, 0.5)` would return the vector `<1.0, 1.0, 1.0, 0.5>`.
pub fn from_hex(code: u32, alpha: f32) -> Vector4<f32> {
    let r = ((code >> 16) & 0xFF) as f32 / 255.0;
    let g = ((code >> 8) & 0xFF) as f32 / 255.0;
    let b = ((code) & 0xFF) as f32 / 255.0;
    Vector4::new(r, g, b, alpha)
}

/// A helper function from Inigo Quilez for quickly generating color palettes.
///
/// Reference: `http://iquilezles.org/www/articles/palettes/palettes.htm`
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

/// Returns the index of the largest component of the vector.
pub fn index_of_largest(v: &Vector4<f32>) -> usize {
    let mut largest_val = v.x.abs();
    let mut largest_index = 0;

    if v.y.abs() > largest_val {
        largest_val = v.y.abs();
        largest_index = 1;
    }
    if v.z.abs() > largest_val {
        largest_val = v.z.abs();
        largest_index = 2;
    }
    if v.w.abs() > largest_val {
        largest_val = v.w.abs();
        largest_index = 3;
    }

    largest_index
}

pub fn saturate(value: f32) -> f32 {
    value.min(1.0).max(0.0)
}

pub fn saturate_between(value: f32, min: f32, max: f32) -> f32 {
    value.min(max).max(min)
}

use std::ops::{Add, Div};

/// Returns the average element of a list of vectors. This is useful for computing
/// cell / face / triangle centroids, for example.
pub fn average<T>(values: &[T], init: &T) -> T
where
    T: Copy + Add<T, Output = T> + Div<f32, Output = T>,
{
    values.iter().fold(*init, |acc, &item| acc + item) / (values.len() as f32)
}

/// Generates an OpenGL shader program based on the source files specified by
/// `vs_path` (vertex shader) and `fs_path` (fragment shader).
pub fn load_shaders(vs_path: &Path, fs_path: &Path) -> Program {
    let mut vs = File::open(vs_path).expect("File not found");
    let mut fs = File::open(fs_path).expect("File not found");

    let mut vs_src = String::new();
    let mut fs_src = String::new();
    vs.read_to_string(&mut vs_src)
        .expect("Something went wrong reading the file");
    fs.read_to_string(&mut fs_src)
        .expect("Something went wrong reading the file");

    Program::new(vs_src, fs_src).unwrap()
}

/// Saves the current frame to disk at `path` with dimensions `width`x`height`.
pub fn save_frame(path: &Path, width: u32, height: u32) {
    let mut pixels: Vec<u8> = Vec::new();
    pixels.reserve((width * height * 3) as usize);

    unsafe {
        // We don't want any alignment padding on pixel rows.
        gl::PixelStorei(gl::PACK_ALIGNMENT, 1);
        gl::ReadPixels(
            0,
            0,
            width as i32,
            height as i32,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            pixels.as_mut_ptr() as *mut c_void,
        );
        pixels.set_len((width * height * 3) as usize);
    }

    image::save_buffer(path, &pixels, width, height, image::RGB(8)).unwrap();
}
