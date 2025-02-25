pub mod wl_app;
pub mod output;
pub mod memory;
pub mod background_image;

use nix::sys::epoll;
use wl_app::WlApp;
use wayland_client::{protocol::{wl_display::WlDisplay, wl_shm}, ConnectError::*, Connection};
use std::{collections::HashMap, os::fd::AsFd, thread, time::{Duration, Instant}};

fn main() {
    let bg_duration = Duration::new(5, 0);
    let epoll = match epoll::Epoll::new(epoll::EpollCreateFlags::empty()) {
        Ok(epoll) => epoll,
        Err(error) => panic!("error when creating an epoll instance: {}", error)
    };

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

    loop {
	let read_guard = event_queue.prepare_read().unwrap();
	let fd = read_guard.connection_fd();

	if let Err(error) = epoll.delete(fd) {
	    match error {
		nix::errno::Errno::ENOENT => (),
		_ => panic!("Error when deleting fd from epoll interest list: {}", error),
	    }
	}

	if let Err(error) = epoll.add(fd, epoll::EpollEvent::new(epoll::EpollFlags::EPOLLIN, 0)) {
	    panic!("Error when adding fd to epoll: {}", error);
	}

	let mut events = [epoll::EpollEvent::empty()];

	if let Err(error) = event_queue.flush() {
	    panic!("error when flushing event queue : {}", error);
	}

	let nfd = match epoll.wait(&mut events, 5000 as u16) {
	    Ok(res) => res,
	    Err(epollerror) => panic!("error when waiting on epoll: {}", epollerror)
	};

	if nfd > 0 {{
	    let n_events = match read_guard.read() {
		Ok(n_events) => n_events,
		Err(error) => {
		    println!("error on read_guard.read: {}", error);
		    continue;
		},
	    };
	    if n_events > 0 {
		match event_queue.dispatch_pending(&mut wl_app) {
		    Ok(_) => (),
		    Err(error) => println!("dispatch event error : {}", error),
		}
	    }
	}}

	let now = Instant::now();
	for (key,output) in wl_app.output_map.iter_mut() {
	    if output.should_update_config == false {
		match output.next_redraw {
		    Some(next_redraw) => {
			if now < next_redraw {
			    continue;
			}
		    },
		    None => (),
		}
		output.render(key, &qh);
		output.next_redraw = Some(now + bg_duration);
	    }
	}
    }
}
