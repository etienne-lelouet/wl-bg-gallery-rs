use std::{fmt::Display, path::PathBuf, sync::mpsc::channel};
use rand::Rng;

use image::{imageops::{overlay, resize}, ImageReader, RgbaImage};

pub enum BackgroundImageError {
    ImageOpenError,
    ImageDecodeError
}

impl Display for BackgroundImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	match self {
	    BackgroundImageError::ImageOpenError => write!(f, "Error when opening image file"),
	    BackgroundImageError::ImageDecodeError => write!(f, "Error when decoding image file"),
	}
    }
}

pub fn fill_buffer_random(buf: &mut[u8]) -> Result<(), BackgroundImageError> {
    println!("in fill_buffer_random");
    let mut rng = rand::rng();
    let red_value = rng.random_range(0..=255);
    let green_value = rng.random_range(0..=255);
    let blue_value = rng.random_range(0..=255);
    let alpha_value = rng.random_range(0..=255);
    for i in (0..buf.len()).step_by(4) {
	buf[i] = blue_value; // blue
	buf[i + 1] = green_value; // green
	buf[i + 2] = red_value; // red
	buf[i + 3] = alpha_value; // alpha
    }
    return Ok(());
}

// TODO: use our defined BackgroundImageError
pub fn open_and_decode_image(path: &PathBuf) -> Option<RgbaImage>{
    let image_buffer = match ImageReader::open(path) {
        Ok(image_buffer) => image_buffer,
        Err(error) => {
	    println!("image open error: {}", error);
	    return None
	},
    };

    let dynamic_image = match image_buffer.decode() {
        Ok(dynamic_image) => dynamic_image,
        Err(error) => {
	    println!("image open error: {}", error);
	    return None
	},
    };

    return Some(dynamic_image.to_rgba8());
}

pub fn downsize_image(image: &RgbaImage, target_width: u32, target_height: u32) -> RgbaImage {
    let (current_width, current_height) = image.dimensions();

    let mut ratio_w: f64 = 0.0;
    if current_width > target_width {
	ratio_w = current_width as f64 / target_width as f64;
    }

    let mut ratio_h: f64 = 0.0;
    if current_height > target_height {
	ratio_h = current_height as f64 / target_height as f64;
    }
    let new_height: u32;
    let new_width: u32;
    if ratio_w > ratio_h {
	new_width = target_width;
	new_height = (current_height as f64 / ratio_w).floor() as u32;
    } else if ratio_h > ratio_w {
	new_width = (current_width as f64 / ratio_h).floor() as u32;
	new_height = target_height;
    } else {
	new_width = target_width;
	new_height = target_height;
    }
    return resize(image, new_width, new_height, image::imageops::FilterType::Lanczos3);

}

pub fn overlay_image(image: &RgbaImage, target_width: u32, target_height: u32) -> RgbaImage {
    let (current_width, current_height) = image.dimensions();
    let mut overlay_img = RgbaImage::new(target_width, target_height);
    let w_offset: i64 = (target_width as i64 - current_width as i64) / 2;
    let h_offset: i64 = (target_height as i64 - current_height as i64) / 2;
    overlay(&mut overlay_img, image, w_offset, h_offset);
    return overlay_img;
}

pub fn fit_image_to_screen(image: RgbaImage, screen_width: u32, screen_height: u32) -> RgbaImage {

    let (current_width, current_height) = image.dimensions();

    let mut new_image: RgbaImage;

    if current_width > screen_width || current_height > screen_height {
	new_image = downsize_image(&image, screen_width, screen_height);
    } else {
	new_image = image;
    }

    let (current_width, current_height) = new_image.dimensions();
    if current_width < screen_width || current_height < screen_height {
	new_image = overlay_image(&new_image, screen_width, screen_height);
    }
    println!("new_image sizes = {:#?}", new_image.dimensions());
    return new_image;
}

pub fn fill_buffer_with_image(
    path: &PathBuf,
    screen_width: u32,
    screen_height: u32,
    buf: &mut[u8]
) -> Result<(), BackgroundImageError> {
    println!("in fill_buffer_with_image");
    let mut image = match open_and_decode_image(path) {
        Some(image) => image,
        None => return Err(BackgroundImageError::ImageOpenError),
    };
    println!("Here !");
    image = fit_image_to_screen(image, screen_width, screen_height);
    println!("{}, {}", image.len(), buf.len());
    assert!(image.len() == buf.len());
    let mut index_in_target = 0;
    for pixel in image.pixels() {
	buf[index_in_target + 0] = pixel[2]; // B
	buf[index_in_target + 1] = pixel[1]; // G
	buf[index_in_target + 2] = pixel[0]; // R
	buf[index_in_target + 3] = pixel[3]; // A
	index_in_target = index_in_target + 4;
    }
    // buf.copy_from_slice(&image);
    return Ok(());
}
