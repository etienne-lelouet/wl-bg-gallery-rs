pub mod wl_app;
pub mod output;
pub mod memory;
use memory::MemoryMapping;
use wl_app::WlApp;
use output::Output;
use wayland_client::{protocol::{wl_display::WlDisplay, wl_shm}, ConnectError::*, Connection};
use std::{collections::HashMap, io::stdin, os::fd::AsFd};
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};

// use std::fs::File;
// use image::ImageReader;

// fn print_file_metadata(file: &File) -> () {
//     match file.metadata() {
//         Ok(metadata) => println!("file len: {}", metadata.len()),
//         Err(err) => println!("Error when fetching metadata : {}", err),
//     }
// }

fn main() {
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

    let mut roundtrip_nr = 3;
    loop {
	println!("making roundtrip {}", roundtrip_nr);
	match event_queue.blocking_dispatch(&mut wl_app) {
	    Ok(_) => println!("roundtrip {} ok !", roundtrip_nr),
	    Err(_) => panic!("roundtrip {} nok", roundtrip_nr),
	};
	roundtrip_nr += 1;
    }
}
