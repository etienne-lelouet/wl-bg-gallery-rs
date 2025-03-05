use std::fs::read_dir;
use std::path::{Path, PathBuf};

use image::{ImageFormat, ImageReader};

#[derive(Debug)]
pub struct ImageFile {
    pub width: u32,
    pub height: u32,
    pub path: PathBuf,
}

impl ImageFile {
    pub fn new(width: u32, height: u32, path: PathBuf) -> Self {
	ImageFile {
	    width,
	    height,
	    path
	}
    }
}

pub fn get_image_list(dir_path: String, authorized_formats: Vec<ImageFormat>) -> Vec<ImageFile> {
    let dir = match read_dir(&dir_path) {
        Ok(dir) => dir,
        Err(error) => panic!("Error when opening {} directory: {}", dir_path, error),
    };

    let mut image_list: Vec<ImageFile> = Vec::new();

    for dir_entry_res in dir {
	let dir_entry = match dir_entry_res {
	    Ok(dir_entry) => dir_entry,
	    Err(error) => {
		println!("error when parsing dir_entry : {}", error);
		continue;
	    },
	};
	let _ = match dir_entry.file_type() {
	    Ok(file_type) => {
		if ! file_type.is_file() {
		    continue;
		}
	    },
	    Err(error) => {
		println!("Could not determine file type for {}: {}", Path::new(&dir_path).join(dir_entry.path()).to_string_lossy(), error);
		continue;
	    }
	};
	let dir_entry_path = Path::new(&dir_path).join(dir_entry.path());

	let mut image_reader = match ImageReader::open(&dir_entry_path) {
	    Ok(image_reader) => image_reader,
	    Err(error) => {
		println!("could not open image {} : {}", Path::new(&dir_path).join(dir_entry.path()).to_string_lossy(), error);
		continue;
	    },
	};
	image_reader = match image_reader.with_guessed_format() {
	    Ok(image_reader) => image_reader,
	    Err(error) => {
		println!("Failed to decode format for file {} : {}", dir_entry_path.to_string_lossy(), error);
		continue;
	    },
	};

	let _ = match image_reader.format() {
	    Some(format) => {
		if ! authorized_formats.contains(&format) {
		    println!("Format {:#?} for file {} is not authorized", format, dir_entry_path.to_string_lossy());
		    continue;
		}
	    },
	    None => {
		println!("Failed to get format for file {}", dir_entry_path.to_string_lossy());
		continue;
	    }
	};

	let (width, height) = match image_reader.into_dimensions() {
	    Ok((width, height)) => (width, height),
	    Err(error)  => {
		println!("Failed to get dimensions format for file {} : {}", dir_entry_path.to_string_lossy(), error);
		continue;
	    },
	};
	image_list.push(ImageFile {
	    width,
	    height,
	    path: dir_entry_path,
	});
    };

    return image_list;
}
