use std::collections::HashMap;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

use image::{ImageFormat, ImageReader};

use crate::image_order_prio::Priority;
use crate::output::Output;

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

struct CurImage {
    image: usize,
    prio: Priority
}

pub fn get_image_fit(image_file: &ImageFile, output: &Output) -> Priority {
    if image_file.width > output.mode_width || image_file.height > output.mode_height {
	return Priority::Upsize { fact: (image_file.width * image_file.height) as f32 / (output.mode_width * output.mode_height) as f32 };
    } else if image_file.width == output.mode_width || image_file.height == output.mode_height {
	return Priority::BestFit;
    } else if image_file.width < output.mode_width && image_file.height < output.mode_height {
	return Priority::Downsize { fact: (image_file.width * image_file.height) as f32 / (output.mode_width * output.mode_height) as f32 };
    }

    // Should never be returned
    return Priority::Any;
}

pub fn assign_everything_everyone(image_list: &Vec<ImageFile>, screens: &mut HashMap<u32, Output>) {
    let index: Vec<usize> = image_list.iter().enumerate().map(|(index, _)| index).collect();
    for (_, screen) in screens.iter_mut() {
	screen.image_list = index.clone();
    }
}

pub fn assign_next_file(image_list: &Vec<ImageFile>, screens: &mut HashMap<u32, Output>) {
    let ratio = image_list.len() / screens.len();
    if ratio == 0 {
	println!("less images than there are screens, assigning everything to everyone");
	assign_everything_everyone(image_list, screens);
	return;
    }
    let mut screens_iter_mut = screens.iter_mut();
    for (index, _file) in image_list.iter().enumerate() {
	let (mut _screen_index, curscreen) = match screens_iter_mut.next() {
	    Some(curscreen) => curscreen,
	    None => {
		screens_iter_mut = screens.iter_mut();
		screens_iter_mut.next().unwrap()
	    }
	};
	curscreen.image_list.push(index);
    }
}

pub fn assign_best_fit(image_list: &Vec<ImageFile>, screens: &mut HashMap<u32, Output>) {
    let ratio = image_list.len() / screens.len();
    if ratio == 0 {
	println!("less images than there are screens, assigning everything to everyone");
	assign_everything_everyone(image_list, screens);
	return;
    }

	cur_best_screen.unwrap().image_list.push(index);
    }
    // let mut marked_indices: HashMap<usize, bool> = HashMap::new();
    // let mut screenstate_map: HashMap<u32, Priority> = HashMap::new();
    // while marked_indices.len() < image_list.len() {

    // 	for (key, output) in screens.iter_mut() {
    // 	    if marked_indices.len() == image_list.len() {
    // 		break;
    // 	    }
    // 	    let current_prio = match screenstate_map.get(key) {
    // 		Some(screenstate) => screenstate,
    // 		None => {
    // 		    screenstate_map.insert(*key, Priority::BestFit);
    // 		    screenstate_map.get(key).unwrap()
    // 		}
    // 	    };

	
    // 	    let mut curimage: Option<CurImage> = None;

    // 	    for (index, image) in image_list.iter().enumerate() {
    // 		if let Some(_)  = marked_indices.get(&index) {
    // 		    continue;
    // 		}

    // 		let prio = get_image_fit(image, output, best_fit_fact);

    // 		if prio == *current_prio {
    // 		    println!("giving image {}x{} to screen {}x{} policy {:#?}", image.width, image.height, output.mode_width, output.mode_height, current_prio);
    // 		    curimage = Some(CurImage {
    // 			image: index,
    // 			prio,
    // 		    });
    // 		    break;
    // 		}
    // 	    }
    // 	    let _ = match curimage {
    // 		Some(image) => {
    // 		    marked_indices.insert(image.image, true);
    // 		    output.image_list.push(image.image);
    // 		}
    // 		None => {
    // 		    let _ = match current_prio {
    // 			Priority::BestFit => screenstate_map.insert(*key, Priority::UpsizeSmall),
    // 			Priority::UpsizeSmall => screenstate_map.insert(*key, Priority::UpsizeAny),
    // 			Priority::UpsizeAny => screenstate_map.insert(*key, Priority::Any),
    // 			Priority::Any => panic!("not supposed to go anywhere after this"),
    // 		    };
    // 		}
    // 	    };
    // 	}
    // }
}

pub fn get_image_list(dir_path: &String, authorized_formats: &Vec<ImageFormat>) -> Vec<ImageFile> {
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
