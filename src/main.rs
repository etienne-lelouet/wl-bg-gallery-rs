pub mod wl_app;
pub mod output;
pub mod memory;
use memory::MemoryMapping;
use wl_app::WlApp;
use wayland_client::{Connection, ConnectError::*, protocol::wl_display::WlDisplay};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

fn print_file_metadata(file: &File) -> () {
    match file.metadata() {
        Ok(metadata) => println!("file len: {}", metadata.len()),
        Err(err) => println!("Error when fetching metadata : {}", err),
    }
}


fn main() {
    // let mut wl_app: wl_app::WlApp = wl_app::WlApp {
    // 	output_map: HashMap::new()
    // };

    // let conn_result = Connection::connect_to_env();
    // let conn: Connection = match conn_result {
    // 	Err(conn_error) => {
    // 	    match conn_error {
    // 		NoWaylandLib => panic!("Connect Error: no wayland lib"),
    // 		InvalidFd => panic!("Connect Error: invalid fd"),
    // 		NoCompositor => panic!("Connect Error: no compositor"),
    // 	    }
    // 	}
    // 	Ok(conn) => conn
    // };

    // // No need for a round-trip, the display object exists implicitly with ID 1
    // let display: WlDisplay = conn.display();

    // let mut event_queue: wayland_client::EventQueue<WlApp> = conn.new_event_queue();
    // let qh = event_queue.handle();

    // display.get_registry(&qh, ());

    // loop {
    // 	match event_queue.roundtrip(&mut wl_app) {
    //         Err(_) => panic!("roundtrip nok"),
    // 	    Ok(_) => {
    // 		println!("Output list !");
    // 		for (key, value) in wl_app.output_map.iter() {
    // 		    println!("{}  {:#?}", key, value);
    // 		}
    // 	    },
    // 	}
    // }
    let mapping : memory::MemoryMapping = match MemoryMapping::new(String::from("ui"), 10) {
        Some(mapping) => mapping,
        None => panic!("failed to map !"),
    };
    let ptr: & mut[u8];
    unsafe {
	ptr = std::slice::from_raw_parts_mut::<u8>(mapping.ptr.as_ptr() as *mut u8, mapping.size);
    }
    println!("len of associated pointer is {}", ptr.len());
    // println!("mapping successful");
    // let mut file: File = File::from(mapping.fd);
    // print_file_metadata(&file);
    // let written = match file.write(b"salut") {
    //     Ok(written) => written,
    //     Err(err) => panic!("Error when writing to file {}", err)
    // };
    // println!("written {} bytes", written);
    // print_file_metadata(&file);
}
