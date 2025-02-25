use image::{imageops::{overlay, resize}, ImageReader, RgbaImage};

pub fn open_and_decode_image(image: &String) -> Option<RgbaImage>{
    let image_buffer = match ImageReader::open(image) {
        Ok(image_buffer) => image_buffer,
        Err(_) => return None,
    };

    let dynamic_image = match image_buffer.decode() {
        Ok(dynamic_image) => dynamic_image,
        Err(_) => return None,
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
    return new_image;
}
