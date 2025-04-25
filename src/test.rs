use std::collections::HashMap;

use crate::{image_file::{self, ImageFile}, output::Output};

pub fn display_list(image_list: &Vec<ImageFile>, screens: &HashMap<u32, Output>) {
    for (_key, screen) in screens.iter() {
	println!("{}x{}", screen.mode_height, screen.mode_width);
	for index in screen.image_list.iter() {
	    let file = image_list.get(*index).unwrap();
	    println!("{}x{}", file.width, file.height);
	}
    }
    println!("");
}

pub fn test_best_fit(image_list: &Vec<ImageFile>, best_fit_fact: f32) {
    let mut screens: HashMap<u32, Output> = HashMap::new();
    let mut output = Output::new();
    output.mode_height = 1080;
    output.mode_width = 1920;
    screens.insert(0, output);
    output = Output::new();
    output.mode_height = 1440;
    output.mode_width = 3440;
    screens.insert(1, output);
    image_file::assign_best_fit(image_list, &mut screens, best_fit_fact);
    display_list(image_list, &screens);
}

pub fn test_next_file(image_list: &Vec<ImageFile>) {
    let mut screens: HashMap<u32, Output> = HashMap::new();
    let mut output = Output::new();
    output.mode_height = 1920;
    output.mode_width = 1080;
    screens.insert(0, output);
    output = Output::new();
    output.mode_height = 1080;
    output.mode_width = 1920;
    screens.insert(1, output);

    image_file::assign_next_file(image_list, &mut screens);
    display_list(image_list, &screens);
}
