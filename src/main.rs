extern crate image;
extern crate num_complex;

use image::{RgbImage, ImageBuffer, Rgb};
use std::fmt::Error;
use num_complex::Complex;
use uuid::Uuid;
use std::ops::Add;

const THRESHHOLD: f64 = 200.0;

fn calculate_color(number: Complex<f64>, iterations: u32) -> Result<Rgb<u8>, Error> {
    let mut z: Complex<f64> = Complex::new(0.0, 0.0);
    let mut cycle = Vec::with_capacity(iterations as usize);

    for i in 0..iterations {
        z = z * z + number;

        if cycle.contains(&z) { return Ok(Rgb::from([0, 0, 0])); }

        if z.norm() > THRESHHOLD {
            return Ok(map_num_to_color(i, iterations));
        }

        cycle.push(z);
    }
    Ok(map_num_to_color(iterations, iterations))
}

fn map_num_to_color(i: u32, maximum: u32) -> Rgb<u8> {
    if i == maximum { return Rgb::from([0, 0, 0]); }
    let frac: f32 = (i as f32 / maximum as f32) * 255.0 * 3.0;

    let r: u8;
    if frac / 3.0 >= 255.0 {
        r = 255
    } else { r = (frac as i32 % 255) as u8 }

    let g: u8;
    if frac / 3.0 >= 255.0 * 2.0 {
        g = 255
    } else { g = (frac as i32 % 255) as u8 }

    let b: u8;
    if frac / 3.0 >= 255.0 * 3.0 {
        b = 255
    } else { b = (frac as i32 % 255) as u8 }

    Rgb::from([r, g, b])
}

fn generate_mandelbrot(width: u32,
                       height: u32,
                       iterations: u32,
                       upper_left: Complex<f64>,
                       lower_right: Complex<f64>) -> Result<RgbImage, Error> {
    let mut out_image: RgbImage = ImageBuffer::new(width, height);

    let section_width: f64 = (upper_left.re - lower_right.re).abs();
    let section_height: f64 = (upper_left.im - lower_right.im).abs();

    for x in 0..width {
        println!("Col: {}", x);
        for y in 0..height {
            let x_offset: f64 = (x as f64 / width as f64) * section_width;
            let y_offset: f64 = (y as f64 / width as f64) * section_height;

            let color = calculate_color(
                Complex::new(upper_left.re + x_offset, upper_left.im + y_offset),
                iterations).unwrap();

            out_image.put_pixel(x, y, color);
        }
    }

    Ok(out_image)
}


fn main() {
    let fractal: RgbImage = generate_mandelbrot(4096,
                                                4096,
                                                100,
                                                Complex::new(-0.7, -0.7),
                                                Complex::new(-0.4, -0.4))
        .unwrap();

    let filename: String = String::from("mandelbrot_")
        .add(&Uuid::new_v4().to_string()) + ".png";

    fractal.save(filename).unwrap();
}
