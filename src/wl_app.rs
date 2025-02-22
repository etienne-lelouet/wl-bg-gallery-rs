use wayland_client::{protocol::{wl_output, wl_registry}, Connection, Dispatch, QueueHandle};
use crate::output::Output;
use std::collections::HashMap;

pub struct WlApp {
    pub output_map: HashMap<u32, Output>,
}

impl Dispatch<wl_output::WlOutput, u32> for WlApp {
    fn event(
        state: &mut Self,
        _proxy: &wl_output::WlOutput,
        event: <wl_output::WlOutput as wayland_client::Proxy>::Event,
        data: &u32,
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
	let output: &mut Output = match state.output_map.get_mut(data) {
	    Some(output) => output,
	    None => panic!("output map does not contain output for key !"),
};
	match event {
	    wl_output::Event::Geometry { x: _, y: _, physical_width: _, physical_height: _, subpixel: _, make, model, transform: _ } => {
		output.make = make;
		output.model = model;

	    },
	    wl_output::Event::Mode { flags: _, width, height, refresh: _ } => {
		output.mode_height = height;
		output.mode_width = width;
	    },
	    wl_output::Event::Done => println!("Done event !"),
	    wl_output::Event::Scale { factor } => println!("Scale event: {}", factor),
	    wl_output::Event::Name { name } => println!("Name event: {}", name),
	    wl_output::Event::Description { description } => {
		output.description = description;
	    },
	    _ => println!("unkown event !")
	}
    }
}


impl Dispatch<wl_registry::WlRegistry, ()> for WlApp {
    fn event(
        state: &mut Self,
        proxy: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _data: &(),
        _conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
	if let wl_registry::Event::Global { name, interface, version } = event {
	    println!("[{}] {} (v{})", name, interface, version);
	    if interface.eq("wl_output") {
		let mut key :u32;
		loop {
		    key = rand::random();
		    if ! state.output_map.contains_key(&key) {
			break;
		    }
		}
		state.output_map.insert(key, Output::new());
		let _wl_output_proxy: wl_output::WlOutput = proxy.bind(name, version, qhandle, key);
	    }
	}
    }
}
