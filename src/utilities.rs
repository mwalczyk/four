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
///
/// In the future, something like this might work better:
///
/// '''
/// fn index_of_largest(values: &[f32]) -> usize {
///    values
///        .iter()
///        .enumerate()
///        .max_by(|&(_, a), &(_, b)| a.partial_cmp(b).unwrap())
///        .unwrap()
///        .0
/// }
/// '''
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

/// Clamps `value` so that it lies in the range `0.0 .. 1.0`.
pub fn saturate(value: f32) -> f32 {
    value.min(1.0).max(0.0)
}

/// Clamps `value` so that it lies in the range `min .. max`.
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

/// Returns the string contents of the file at `path`.
pub fn load_file_as_string(path: &Path) -> String {
    let mut file = File::open(path).expect("File not found");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Something went wrong reading the file");

    contents
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
