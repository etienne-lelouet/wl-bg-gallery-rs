pub mod wl_app;
pub mod output;
pub mod memory;
pub mod background_image;
pub mod image_file;

use nix::sys::epoll;
use wl_app::WlApp;
use wayland_client::{protocol::{wl_display::WlDisplay, wl_shm}, ConnectError::*, Connection};
use std::{collections::HashMap, time::{Duration, Instant}};

fn main() {
    let image_list = image_file::get_image_list(String::from("/home/etienne/Pictures/wallpaper/"));
    if image_list.len() == 0 {
	panic!("no images to set as background !");
    }

    let bg_duration_as_duration = Duration::new(15, 0);
    let mut next_timer = None;
    let bg_duration_as_epoll_timer = match epoll::EpollTimeout::try_from(bg_duration_as_duration.as_millis()) {
        Ok(duration) => duration,
        Err(error) => panic!("Time between updates is incorrect: {}", error),
    };
    let acceptable_delta = Duration::new(1, 0);

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

    let epoll = match epoll::Epoll::new(epoll::EpollCreateFlags::empty()) {
	Ok(epoll) => epoll,
	Err(error) => panic!("error when creating an epoll instance: {}", error)
    };

    let mut index_in_image_array = 0;

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

	if let None = next_timer {
	    next_timer = Some(bg_duration_as_duration);
	}

	let timeout = match epoll::EpollTimeout::try_from(next_timer.unwrap()) {
	    Ok(timeout) => timeout,
	    Err(_) => bg_duration_as_epoll_timer,
	};

	println!("epoll wait for {:#?}", next_timer.unwrap());

	let nfd = match epoll.wait(&mut events, timeout) {
	    Ok(res) => res,
	    Err(epollerror) => panic!("error when waiting on epoll: {}", epollerror)
	};
	println!("#########################################epoll wait finished#########################################");
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
	next_timer = None;
	for (key,output) in wl_app.output_map.iter_mut() {
	    if output.should_update_config == false {
		match output.next_redraw {
		    Some(next_redraw) => {
			if now + acceptable_delta < next_redraw {
			    let next_redraw_delta = next_redraw - now;
			    println!("output {}, next_redraw_delta: {:#?}", output.name, next_redraw_delta);
			    if let None = next_timer {
				next_timer = Some(next_redraw_delta);
			    }
			    if next_redraw_delta < next_timer.unwrap() {
				next_timer = Some(next_redraw_delta);
			    }
			    continue;
			}
		    },
		    None => (),
		}

		output.render(key, &qh, Some(&(image_list.get(index_in_image_array).unwrap().path)));
		output.next_redraw = Some(Instant::now() + bg_duration_as_duration);
		index_in_image_array = (index_in_image_array + 1) % image_list.len();
	    }
	}
    }
}
