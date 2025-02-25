pub mod wl_app;
pub mod output;
pub mod memory;
pub mod background_image;

use background_image::{fit_image_to_screen, open_and_decode_image};
use timer::Timer;
use wl_app::WlApp;
use wayland_client::{protocol::{wl_display::WlDisplay, wl_shm}, ConnectError::*, Connection};
use std::{collections::HashMap, thread, time::Duration};

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

    let timer = Timer::new();

    let mut roundtrip_nr = 3;
    loop {
	println!("making roundtrip {}", roundtrip_nr);
	match event_queue.roundtrip(&mut wl_app) {
	    Ok(_) => println!("roundtrip {} ok !", roundtrip_nr),
	    Err(_) => panic!("roundtrip {} nok", roundtrip_nr),
	};
	for (key, output) in wl_app.output_map.iter_mut() {
	    if output.should_update_config == false { // && no callback scheduled
		output.render(key, &qh);
	    }
	}
	roundtrip_nr += 1;
	thread::sleep(Duration::new(5, 0));
    }
}
