use std::{ops::Deref, path::PathBuf};

use image::{DynamicImage, ImageBuffer, ImageError, Luma, Pixel, Rgb};

static default_pixel: Rgb<u8> = Rgb([0u8, 0u8, 0u8]);

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: seam-carving <path to image>");
        return;
    }
    let path_str = args[1].as_str();
    let path = PathBuf::from(path_str);
    if !path.exists() {
            eprintln!("no file found at \"{}\"", path_str);
            return;
        }
    let mut image = match read_image_file(path.clone()) {
                Ok(image) => image,
                Err(e) => {
                    eprintln!("could not load image: {}", e);
                    return;
                }
            };
    let mut image = image.into_rgb8();
    let mut gradient: ImageBuffer<Luma<u8>, Vec<_>> = ImageBuffer::new(image.width(), image.height());
    for x in 0..image.width() {
        for y in 0..image.height() {
            gradient.get_pixel_mut(x, y).0[0] = gradient_magnitude(&mut image, x, y);
        }
    }
    let mut file_name = path.file_name().unwrap().to_str().unwrap().to_owned();
    file_name.truncate(file_name.as_str().rfind('.').unwrap());
    file_name.push_str("_gradient.png");
    match gradient.save(path.with_file_name(file_name.clone())){
        Ok(_) => println!("Image saved as {}", file_name),
        Err(e) => eprintln!("Failed to save image: {}", e),
    };
}

fn read_image_file(path: PathBuf) -> Result<DynamicImage, ImageError> {
    image::io::Reader::open(path)?.decode()
}

fn pixel_magnitude(pixel: Option<&Rgb<u8>>) -> f32 {
    match pixel {
        None => 0.0,
        Some(pixel) => {
            let [r, g, b] = pixel.0;
            return f32::sqrt(r as f32 * r as f32 + g as f32 * g as f32 + b as f32 * b as f32);
        }
    }
}

fn gradient_magnitude(image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, x: u32, y: u32) -> u8 {
    let gradient_x = pixel_magnitude(image.get_pixel_checked(x.wrapping_add(1), y)) - pixel_magnitude(image.get_pixel_checked(x.wrapping_sub(1), y));
    let gradient_y = pixel_magnitude(image.get_pixel_checked(x, y.wrapping_add(1))) - pixel_magnitude(image.get_pixel_checked(x, y.wrapping_sub(1)));
    return f32::sqrt(gradient_x * gradient_x + gradient_y * gradient_y) as u8;
}