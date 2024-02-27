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
    //storing u8 values inside an i16, in order to capture the full range and still have negatives available
    let mut gradient: ImageBuffer<Luma<i16>, Vec<_>> = ImageBuffer::new(image.width(), image.height());
    for x in 0..image.width() {
        for y in 0..image.height() {
            gradient.put_pixel(x, y, Luma([gradient_magnitude(&mut image, x, y) as i16]));
        }
    }

    let mut seams: ImageBuffer<Luma<i32>, Vec<i32>> = ImageBuffer::new(image.width(), image.height());

    //initialize the first row to have something to work with
    for i in 0..seams.width() {
        seams.get_pixel_mut(i, seams.height() - 1).0[0] = gradient.get_pixel(i, gradient.height() - 1).0[0] as i32;
    }

    //count vertical seams, from bottom to top
    for y in (0..(gradient.height()-2)).rev(){
        for x in 0..gradient.width() {
            let current = pixel_value(gradient.get_pixel_checked(x, y));
            let left = pixel_value(gradient.get_pixel_checked(x.wrapping_sub(1), y.wrapping_add(1)));
            let center = pixel_value(gradient.get_pixel_checked(x, y.wrapping_add(1)));
            let right = pixel_value(gradient.get_pixel_checked(x.wrapping_add(1), y.wrapping_add(1)));
            //println!("current: {}, left: {}, center: {}, right: {}", current, left, center, right);
            if left < right && left < center {
                gradient.get_pixel_mut_checked(x.wrapping_sub(1), y.wrapping_add(1)).map(|x| *x = Luma([i32::MAX as i16]));
                seams.put_pixel(x, y, Luma([current + left]))
            }
            else if right < left && right < center {
                gradient.get_pixel_mut_checked(x.wrapping_add(1), y.wrapping_add(1)).map(|x| *x = Luma([i32::MAX as i16]));
                seams.put_pixel(x, y, Luma([current + right]))
            }
            else {
                gradient.get_pixel_mut_checked(x, y.wrapping_add(1)).map(|x: &mut Luma<i16>| *x = Luma([i32::MAX as i16]));
                seams.put_pixel(x, y, Luma([current + center]))
            }
        }
    }

    let seam_count = 200;

    
    let mut seam_values = vec![i32::MAX; seam_count];
    let mut seam_indices = vec![0u32; seam_count];

    for x in 0..seams.width() {
        let pixel = seams.get_pixel(x, 0).0[0];
        'index: for i in 0..seam_count {
            if pixel < seam_values[i] {
                //println!("found seam at {}", x);
                seam_values[i] = pixel;
                seam_indices[i] = x;
                break 'index;
            }
        }
    }

    for i in 0..seam_count {
        for y in 0..seams.height() {
            image.put_pixel(seam_indices[i], y, Rgb([0, 255, 0]));
            let center = i32::abs(i32::abs(seams.get_pixel_checked(seam_indices[i], y + 1).unwrap_or(&Luma([i32::MAX])).0[0] - seams.get_pixel(seam_indices[i], y).0[0]) - i32::abs(gradient.get_pixel(seam_indices[i], y).0[0] as i32));
            let left = i32::abs(i32::abs(seams.get_pixel_checked(seam_indices[i].wrapping_sub(1), y + 1).unwrap_or(&Luma([i32::MAX])).0[0] - seams.get_pixel(seam_indices[i], y).0[0]) - i32::abs(gradient.get_pixel(seam_indices[i], y).0[0] as i32));
            let right = i32::abs(i32::abs(seams.get_pixel_checked(seam_indices[i].wrapping_add(1), y + 1).unwrap_or(&Luma([i32::MAX])).0[0] - seams.get_pixel(seam_indices[i], y).0[0]) - i32::abs(gradient.get_pixel(seam_indices[i], y).0[0] as i32));

            if left < center && left < right {
                seam_indices[i] -= 1;
            }
            else if right < center && right < left {
                seam_indices[i] += 1;
            }
            seams.put_pixel(seam_indices[i], y, Luma([i32::MAX/2]));
        }
    }

    let mut file_name = path.file_name().unwrap().to_str().unwrap().to_owned();
    file_name.truncate(file_name.as_str().rfind('.').unwrap());
    file_name.push_str("_seams.png");
    match image.save(path.with_file_name(file_name.clone())){
        Ok(_) => println!("Image saved as {}", file_name),
        Err(e) => eprintln!("Failed to save image: {}", e),
    };
}

fn read_image_file(path: PathBuf) -> Result<DynamicImage, ImageError> {
    image::io::Reader::open(path)?.decode()
}

fn pixel_value(pixel: Option<&Luma<i16>>) -> i32 {
    match pixel {
        None => i32::MAX,
        Some(pixel) => pixel.0[0] as i32
    }
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

fn gradient_magnitude(image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, x: u32, y: u32) -> i16 {
    let gradient_x = pixel_magnitude(image.get_pixel_checked(x.wrapping_add(1), y)) - pixel_magnitude(image.get_pixel_checked(x.wrapping_sub(1), y));
    let gradient_y = pixel_magnitude(image.get_pixel_checked(x, y.wrapping_add(1))) - pixel_magnitude(image.get_pixel_checked(x, y.wrapping_sub(1)));
    return f32::sqrt(gradient_x * gradient_x + gradient_y * gradient_y) as u8 as i16;
}