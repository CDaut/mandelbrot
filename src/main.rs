extern crate image;
extern crate num_complex;

use image::{RgbImage, ImageBuffer, Rgb};
use std::fmt::Error;
use num_complex::Complex;
use indicatif::ProgressBar;
use rayon::prelude::*;

const THRESHHOLD: f64 = 200_i32.pow(2) as f64;

struct PixelInfo {
    x: u32,
    y: u32,
    color: Rgb<u8>,
}

fn calculate_color(number: Complex<f64>, iterations: u32) -> Result<Rgb<u8>, Error> {
    let mut z: Complex<f64> = Complex::new(0.0, 0.0);
    let mut cycle = Vec::with_capacity(iterations as usize);

    for i in 0..iterations {
        z = z * z + number;

        if cycle.contains(&z) { return Ok(Rgb::from([0, 0, 0])); }

        if z.norm_sqr() > THRESHHOLD {
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
    let g: u8;
    let b: u8;

    if frac < 255.0 {
        r = frac.ceil() as u8;
        g = 0;
        b = 0;
    } else if frac >= 255.0 && frac < 255.0 * 2.0 {
        r = 255;
        g = (frac as i32 % 255) as u8;
        b = 0;
    } else if frac >= 255.0 * 2.0 && frac < 255.0 * 3.0 {
        r = 255;
        g = 255;
        b = (frac as i32 % 255) as u8;
    } else {
        r = 255;
        g = 255;
        b = 255;
    }

    Rgb::from([b, g, r])
}

fn generate_mandelbrot(width: u32,
                       height: u32,
                       iterations: u32,
                       upper_left: Complex<f64>,
                       lower_right: Complex<f64>) -> Result<RgbImage, Error> {
    let mut out_image: RgbImage = ImageBuffer::new(width, height);

    let section_width: f64 = (upper_left.re - lower_right.re).abs();
    let section_height: f64 = (upper_left.im - lower_right.im).abs();

    let bar = ProgressBar::new(width as u64);

    let mut all_pixels: Vec<Vec<PixelInfo>> = vec![];

    (0..width).into_par_iter().map(|x| {
        bar.inc(1);
        let mut row: Vec<PixelInfo> = vec![];

        for y in 0..height {
            let x_offset: f64 = (x as f64 / width as f64) * section_width;
            let y_offset: f64 = (y as f64 / width as f64) * section_height;

            let color = calculate_color(
                Complex::new(upper_left.re + x_offset, upper_left.im + y_offset),
                iterations).unwrap();
            row.push(PixelInfo {
                x,
                y,
                color,
            });
        }
        row
    }).collect_into_vec(&mut all_pixels);

    for vector in all_pixels {
        for pixel in vector {
            out_image.put_pixel(pixel.x, pixel.y, pixel.color);
        }
    }


    bar.finish_with_message("Rendered frame.");
    Ok(out_image)
}

fn render_around_point(image_size: u32,
                       iterations: u32,
                       window_size: f64,
                       pivot_point: Complex<f64>) -> Result<RgbImage, Error> {
    let upper_left: Complex<f64> = Complex::new(pivot_point.re - window_size / 2.0,
                                                pivot_point.im - window_size / 2.0);

    let lower_right: Complex<f64> = Complex::new(pivot_point.re + window_size / 2.0,
                                                 pivot_point.im + window_size / 2.0);

    generate_mandelbrot(image_size,
                        image_size,
                        iterations,
                        upper_left,
                        lower_right)
}

fn render_sequence(end_frame: u32,
                   start_frame: u32,
                   image_size: u32,
                   iterations: u32,
                   initial_size: f64,
                   zoom_point: Complex<f64>,
                   scale_factor: f32,
                   path: String) {
    let mut scale = initial_size * (1.0 - scale_factor).powi(start_frame as i32) as f64;

    for frame in start_frame..end_frame {
        let fractal: RgbImage = render_around_point(image_size,
                                                    iterations,
                                                    scale,
                                                    zoom_point)
            .unwrap();

        scale *= (1.0 - scale_factor) as f64;

        let filename: String = path.clone() + "frame_" + &*frame.to_string() + ".png";
        fractal.save(filename).unwrap();
        println!("Rendered frame {}", frame);
    }
}

fn main() {
    render_sequence(240,
                    0,
                    500,
                    200,
                    2.0,
                    Complex::new(-0.761574, -0.0847596),
                    1.0 / 10.0,
                    "./out/".parse().unwrap(),
    );
}
