pub mod wl_app;
pub mod output;
pub mod memory;
use memory::MemoryMapping;
use wl_app::WlApp;
use output::Output;
use wayland_client::{protocol::{wl_display::WlDisplay, wl_shm}, ConnectError::*, Connection};
use std::{collections::HashMap, io::stdin, os::fd::AsFd};
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};
use rand::Rng;

// use std::fs::File;
// use image::ImageReader;

// fn print_file_metadata(file: &File) -> () {
//     match file.metadata() {
//         Ok(metadata) => println!("file len: {}", metadata.len()),
//         Err(err) => println!("Error when fetching metadata : {}", err),
//     }
// }

fn fill_buffer_random(buf: &mut[u8], stride: u32, height: u32) -> &mut[u8] {
    let mut rng = rand::rng();
    let gcd = num::integer::gcd(stride, height);
    for i in (0..buf.len()).step_by((gcd * 4) as usize) {
	let red_value = rng.random_range(0..255);
	let green_value = rng.random_range(0..255);
	let blue_value = rng.random_range(0..255);
	for ii in (0..gcd).step_by(4) {
	    buf[i + ii as usize] = 255;
	    buf[i + ii as usize + 1] = red_value;
	    buf[i + ii as usize + 2] = green_value;
	    buf[i + ii as usize + 2] = blue_value;
	}
    }
    return buf;
}

fn main() {
    let mut rng = rand::rng();
    let mut wl_app: wl_app::WlApp = wl_app::WlApp {
	output_map: HashMap::new(),
	supported_formats_vec: Vec::new(),
	wl_shm: None,
	compositor_proxy: None,
	wlr_layer_shell_proxy: None
    };

    let conn_result = Connection::connect_to_env();
    let conn: Connection = match conn_result {
	Err(conn_error) => {
	    match conn_error {
		NoWaylandLib => panic!("Connect Error: no wayland lib"),
		InvalidFd => panic!("Connect Error: invalid fd"),
		NoCompositor => panic!("Connect Error: no compositor"),
	    }
	}
	Ok(conn) => conn
    };

    // No need for a round-trip, the display object exists implicitly with ID 1
    let display: WlDisplay = conn.display();

    let mut event_queue: wayland_client::EventQueue<WlApp> = conn.new_event_queue();
    let qh = event_queue.handle();

    display.get_registry(&qh, ());

    println!("parsing globals");

    match event_queue.roundtrip(&mut wl_app) {
        Err(_) => panic!("roundtrip 1 nok"),
	Ok(_) => println!("roundtrip 1 ok"),
    }


    println!("parsing screen conf and supported formats");

    match event_queue.roundtrip(&mut wl_app) {
        Err(_) => panic!("roundtrip 2 nok"),
	Ok(_) => println!("roundtrip 2 ok"),
    }

    if wl_app.supported_formats_vec.iter().any(|format| {
	match format {
	    wl_shm::Format::Argb8888 =>  return true,
	    _ => return false,
}
    }) == false {
	panic!("could not find Argb8888 in supported formats vec");
    }

    println!("creating surface for screen");

    let output_key: u32;

    match wl_app.compositor_proxy {
	Some(ref proxy) => {
	    let output_ref: &mut Output;
	    match wl_app.output_map.iter_mut().next() {
		Some((first_key, output)) => {
		    output_key = *first_key;
		    output_ref = output;
		},
		None => panic!("could not find screen"),
	    };
	    output_ref.wl_surface_proxy = Some(proxy.create_surface(&qh, ()));
	    let surface_proxy = output_ref.wl_surface_proxy.as_ref().unwrap();
	    let region = wl_app.compositor_proxy.as_ref().unwrap().create_region(&qh, ());
	    surface_proxy.set_input_region(Some(&region));
	    region.destroy();
	    surface_proxy.commit();
	},
	None => {
	    panic!("Should have a compositor proxy");
	}
    }


    match event_queue.roundtrip(&mut wl_app) {
        Ok(_) => println!("roundtrip 3 ok"),
        Err(_) => panic!("roundtrip 3 nok"),
    };

    match wl_app.output_map.get_mut(&output_key) {
	Some(output) => {
	    let surface_proxy = output .wl_surface_proxy.as_ref().unwrap();
	    let layer_shell = wl_app.wlr_layer_shell_proxy.as_ref().unwrap();
	    output.wlr_layer_surface_proxy = Some(
		layer_shell.get_layer_surface(
		    &(output.wl_surface_proxy.as_ref().unwrap()),
		    Some(&(output.wl_output_proxy.as_ref().unwrap())),
		     zwlr_layer_shell_v1::Layer::Background,
		     String::from("wallpaper"),
		     &qh,
		     output_key
		)
	    );
	    surface_proxy.commit();
	}
	None => panic!("no output for key")
    }

    match event_queue.roundtrip(&mut wl_app) {
        Ok(_) => println!("roundtrip 3 ok"),
        Err(_) => panic!("roundtrip 3 nok"),
    };

    // println!("creating layer_surface from surface");

    match wl_app.output_map.get_mut(&output_key) {
	Some(output) => {
	    let layer_surface_proxy = output.wlr_layer_surface_proxy.as_ref().unwrap();
	    layer_surface_proxy.set_size(0, 0);
	    layer_surface_proxy.set_anchor(zwlr_layer_surface_v1::Anchor::all());
	    layer_surface_proxy.set_exclusive_zone(-1);
	    // layer_surface_proxy.set_margin(0, 0, 0, 0);
	}
	None => panic!("no output for key")
    }

    match event_queue.roundtrip(&mut wl_app) {
        Ok(_) => println!("roundtrip 5 ok !"),
        Err(_) => panic!("roundtrip 5 nok"),
    };

    println!("configuring shm_pool for output");

    match wl_app.output_map.get_mut(&output_key) {
	Some(output) => {
	    let shm_pool_size = 3840 * 2160 * 4;
	    output.mapping = match MemoryMapping::new(String::from("ui"), shm_pool_size as usize) {
		Some(mapping) => Some(mapping),
		None => panic!("Creating buffer failed !"),
	    };
	    output.wl_shm_pool =  Some(wl_app.wl_shm.as_ref().unwrap().create_pool(output.mapping.as_ref().unwrap().fd.as_fd(), shm_pool_size, &qh, output_key))
	},
	None => panic!("no output for key")
    }

    match event_queue.roundtrip(&mut wl_app) {
        Ok(_) => println!("roundtrip 6 ok !"),
        Err(_) => panic!("roundtrip 6 nok"),
    };

    println!("creating buffer from shm_pool");
    match wl_app.output_map.get_mut(&output_key) {
        Some(output) => {
	    output.wl_buffer = Some(output.wl_shm_pool.as_ref().unwrap().create_buffer(0, output.mode_width, output.mode_height, output.mode_width * 4, wl_shm::Format::Argb8888, &qh, output_key))
	},
	None => panic!("no output for key")
    }

    println!("filling buffer, and attaching it");
    match wl_app.output_map.get_mut(&output_key) {
        Some(output) => {
	    let buffer = output.wl_buffer.as_ref().unwrap();
	    let ptr: & mut[u8];
	    let mapping = output.mapping.as_ref().unwrap();
	    unsafe {
		ptr = std::slice::from_raw_parts_mut::<u8>(mapping.ptr.as_ptr() as *mut u8, mapping.size);
	    }
	    fill_buffer_random(ptr, output.mode_width as u32, output.mode_height as u32);
	    // for index in (0..mapping.size).step_by(4) {
	    // 	ptr[index] = 255;
	    // 	ptr[index + 1] = rng.random_range(0..255);
	    // 	ptr[index + 2] = rng.random_range(0..255);
	    // 	ptr[index + 3] = rng.random_range(0..255);
	    // }
	    let surface = output.wl_surface_proxy.as_ref().unwrap();
	    surface.set_buffer_scale(1);
	    surface.attach(Some(buffer), 0, 0);
	    surface.damage_buffer(0, 0, i32::MAX, i32::MAX);
	    surface.commit();
	},
	None => panic!("no output for key")
    }

    match event_queue.roundtrip(&mut wl_app) {
        Ok(_) => println!("roundtrip 7 ok !"),
        Err(_) => panic!("roundtrip 7 nok"),
    };

    let mut user_input = String::new();
    let mut roundtrip_nr = 8;

    loop {
	println!("waiting for input");
	match stdin().read_line(&mut user_input) {
	    Ok(_) => println!("user_input: {}", user_input),
	    Err(err) => panic!("IO error when reding on stdin: {}", err),
	}
	println!("here");
	let ptr: & mut[u8];
	let output = wl_app.output_map.get_mut(&output_key).unwrap();
	let mapping = output.mapping.as_ref().unwrap();
	unsafe {
	    ptr = std::slice::from_raw_parts_mut::<u8>(mapping.ptr.as_ptr() as *mut u8, mapping.size);
	}
	fill_buffer_random(ptr, output.mode_width as u32, output.mode_height as u32);
	// for index in (0..mapping.size).step_by(4) {
	//     ptr[index] = 255;
	//     ptr[index + 1] = rng.random_range(0..255);
	//     ptr[index + 2] = rng.random_range(0..255);
	//     ptr[index + 3] = rng.random_range(0..255);
	// };
	let surface = output.wl_surface_proxy.as_ref().unwrap();
	output.wl_buffer = Some(output.wl_shm_pool.as_ref().unwrap().create_buffer(0, output.mode_width, output.mode_height, output.mode_width * 4, wl_shm::Format::Argb8888, &qh, output_key));
	surface.attach(Some(output.wl_buffer.as_ref().unwrap()), 0, 0);
	surface.damage_buffer(0, 0, i32::MAX, i32::MAX);
	surface.commit();

	match event_queue.roundtrip(&mut wl_app) {
            Ok(_) => println!("roundtrip {} ok !", roundtrip_nr),
            Err(_) => panic!("roundtrip {} nok", roundtrip_nr),
	};
	roundtrip_nr += 1;
    }
}
