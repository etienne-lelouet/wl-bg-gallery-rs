use nix::sys::epoll;
use wayland_client::{protocol::{wl_buffer, wl_compositor, wl_output, wl_region, wl_registry, wl_shm, wl_shm_pool, wl_surface}, ConnectError, Connection, Dispatch, EventQueue, QueueHandle};
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1;
use crate::{config::Config, image_file::ImageFile, output::Output};
use std::{collections::HashMap, time::{Duration, Instant}};

pub struct WlApp {
    pub output_map: HashMap<u32, Output>,
    pub compositor_proxy: Option<wl_compositor::WlCompositor>,
    pub wlr_layer_shell_proxy: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    pub wl_shm: Option<wl_shm::WlShm>,
    pub supported_formats_vec: Vec<wl_shm::Format>,
    pub config: Config,
    pub image_list: Vec<ImageFile>
}

impl Dispatch<wl_shm::WlShm, ()> for WlApp {
    fn event(
        state: &mut Self,
        _proxy: &wl_shm::WlShm,
        event: <wl_shm::WlShm as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
	match event {
	    wl_shm::Event::Format { format } => {
		match format {
		    wayland_client::WEnum::Value(value) => state.supported_formats_vec.push(value),
		    wayland_client::WEnum::Unknown(unkown) => println!("WlShm format event : unkown pixel format format {}", unkown),
		}
	    },
	    _ => println!("wl_shm unkown event !"),
}
    }
}
impl Dispatch<wl_region::WlRegion, ()> for WlApp {
    fn event(
        _state: &mut Self,
        _proxy: &wl_region::WlRegion,
        _event: <wl_region::WlRegion as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        println!("should not receive event for wl region !")
    }
}
impl Dispatch<wl_buffer::WlBuffer, u32> for WlApp {
    fn event(
        state: &mut Self,
        _proxy: &wl_buffer::WlBuffer,
        event: <wl_buffer::WlBuffer as wayland_client::Proxy>::Event,
        data: &u32,
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
	match event {
	    wl_buffer::Event::Release => {
		let output = state.output_map.get_mut(data).unwrap();
		println!("buffer release event for output {}", output.name);
		if let Some(buffer) = output.wl_buffer.as_ref() {
		    buffer.destroy();
		    output.wl_buffer = None;
		}

	    },
	    _ => println!("unkown event for wl_buffer"),
	}
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, u32> for WlApp {
    fn event(
        _state: &mut Self,
        _proxy: &wl_shm_pool::WlShmPool,
        _event: <wl_shm_pool::WlShmPool as wayland_client::Proxy>::Event,
        _data: &u32,
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        println!("should not receive event for shm pool !")
    }
}

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, u32> for WlApp {
    fn event(
        state: &mut Self,
        proxy: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: <zwlr_layer_surface_v1::ZwlrLayerSurfaceV1 as wayland_client::Proxy>::Event,
        data: &u32,
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
	match event {
	    zwlr_layer_surface_v1::Event::Configure { serial, width, height } => {
		let output = state.output_map.get_mut(&data).unwrap();
		println!("Configure event for output {}, serial {}, width: {}, height: {}", output.name, serial, width, height);
		if output.should_update_config && output.wlr_layer_surface_proxy.is_some() && output.wl_surface_proxy.is_some(){
		    proxy.ack_configure(serial);
		    output.should_update_config = false;
		}
	    },
	    zwlr_layer_surface_v1::Event::Closed => println!("close event !"),
	    _ => todo!(),
}
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for WlApp {
    fn event(
        _state: &mut Self,
        _proxy: &wl_compositor::WlCompositor,
        _event: <wl_compositor::WlCompositor as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
	panic!("not supposed to receive event for compositor proxy !");
    }
}

impl Dispatch<wl_output::WlOutput, u32> for WlApp {
    fn event(
        state: &mut Self,
	_proxy: &wl_output::WlOutput,
        event: <wl_output::WlOutput as wayland_client::Proxy>::Event,
        data: &u32,
        _conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
	let output: &mut Output = match state.output_map.get_mut(data) {
	    Some(output) => output,
	    None => panic!("output map does not contain output for key !"),
	};
	match event {
	    wl_output::Event::Geometry { x: _, y: _, physical_width: _, physical_height: _, subpixel: _, make, model, transform: _ } => {
		println!("geometry event");
		output.make = make;
		output.model = model;

	    },
	    wl_output::Event::Mode { flags: _, width, height, refresh: _ } => {
		println!("mode event ours: {}x{} new: {}x{}", output.mode_width, output.mode_height, width, height);
		// unchecked cast but if the mode event contains a negative number, we have biger issues
		if output.mode_height != height as u32 {
		    output.mode_height = height as u32;
		    output.should_update_config = true;
		}

		if output.mode_width != width as u32 {
		    output.mode_width = width as u32;
		    output.should_update_config = true;
		}
		if output.should_update_config {
		    output.clear();
		}
	    },
	    wl_output::Event::Done => {
		println!("Done event for for {}, getting and configuring surface and shm", output.name);
		let compositor_proxy = state.compositor_proxy.as_ref().unwrap();
		if ! output.should_update_config {
		    println!("received done event for surface that should not be updated");
		    return;
		}
		output.configure_shm_pool(data, state.wl_shm.as_ref().unwrap(), qhandle);
		output.wl_surface_proxy = Some(compositor_proxy.create_surface(qhandle, ()));
		let surface_proxy = output.wl_surface_proxy.as_ref().unwrap();
		let region = compositor_proxy.create_region(qhandle, ());
		surface_proxy.set_input_region(Some(&region));
		region.destroy();
		surface_proxy.commit();
		if let Some(ref layer_shell) = output.wlr_layer_surface_proxy {
		    layer_shell.destroy();
		}
		let layer_shell = state.wlr_layer_shell_proxy.as_ref().unwrap();
		output.wlr_layer_surface_proxy = Some(
		    layer_shell.get_layer_surface(
			&(output.wl_surface_proxy.as_ref().unwrap()),
			Some(&(output.wl_output_proxy.as_ref().unwrap())),
			zwlr_layer_shell_v1::Layer::Background,
			String::from("wallpaper"),
			qhandle,
			*data
		    )
		);
		let layer_surface_proxy = output.wlr_layer_surface_proxy.as_ref().unwrap();
		layer_surface_proxy.set_size(output.mode_width, output.mode_height);
		layer_surface_proxy.set_anchor(zwlr_layer_surface_v1::Anchor::all());
		layer_surface_proxy.set_exclusive_zone(-1);
		println!("initial commit for surface");
		surface_proxy.commit();
	    }
	    wl_output::Event::Scale { factor } => println!("scale event : {}", factor),
	    wl_output::Event::Name { name } => {
		println!("name event {name}");
		output.name = name;
	},
	    wl_output::Event::Description { description } => {
		println!("description event {description}");
		output.description = description;
	    },
	    _ => println!("unkown event !")
	}
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for WlApp {
    fn event(
        _state: &mut Self,
        _proxy: &wl_surface::WlSurface,
        event: <wl_surface::WlSurface as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
	match event {
	    wl_surface::Event::Enter { output: _ } => println!("Surface enter event !"),
	    wl_surface::Event::Leave { output: _ } => println!("Surface leave event !"),
	    wl_surface::Event::PreferredBufferScale { factor } => println!("Preferred buffer scale event : {}", factor),
	    wl_surface::Event::PreferredBufferTransform { transform: _ } => println!("Preferred buffer transfor event :"),
	    _ => println!("Unkown event !"),
	}
    }
}

impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for WlApp {
    fn event(
        _state: &mut Self,
        _proxy: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
        _event: <zwlr_layer_shell_v1::ZwlrLayerShellV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
	panic!("not supposed to receive event for wlr layer shell proxy !");
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
	match event {
	    wl_registry::Event::Global { name, interface, version } => {
		if interface.eq("wl_output") {
		    println!("[{}] {} (v{})", name, interface, version);
		    let mut output = Output::new();
		    output.wl_output_proxy = Some(proxy.bind(name, version, qhandle, name));
		    state.output_map.insert(name, output);
		}

		if interface.eq("wl_compositor") {
		    println!("[{}] {} (v{})", name, interface, version);
		    state.compositor_proxy = Some(proxy.bind(name, version, qhandle, ()));
		}

		if interface.eq("zwlr_layer_shell_v1") {
		    println!("[{}] {} (v{})", name, interface, version);
		    state.wlr_layer_shell_proxy = Some(proxy.bind(name, version, qhandle, ()));
		}

		if interface.eq("wl_shm") {
		println!("[{}] {} (v{})", name, interface, version);
		    state.wl_shm = Some(proxy.bind(name, version, qhandle, ()));
		}
	    },
	    wl_registry::Event::GlobalRemove { name } => {
		let output = match state.output_map.get_mut(&name) {
		    Some(output) => output,
		    None => {
			println!("global_remove does not concern a screen not found with key");
			return;
		    },
		};
		println!("destroying screen {}", output.name);
		output.clear();
		state.output_map.remove(&name);
	    },
	    _ => println!("unkown event for wl_registry")
	}
    }
}

impl WlApp {
    pub fn new(config: Config, image_list: Vec<ImageFile>) -> WlApp {
	WlApp {
	    output_map: HashMap::new(),
	    supported_formats_vec: Vec::new(),
	    wl_shm: None,
	    compositor_proxy: None,
	    wlr_layer_shell_proxy: None,
	    config,
	    image_list
	}
    }

    pub fn run(&mut self) {
	let mut event_queue = self.setup();
	self.main_loop(&mut event_queue);
    }

    pub fn setup(&mut self) -> EventQueue<WlApp> {
	let conn_result = Connection::connect_to_env();
	let connection: Connection = match conn_result {
	    Err(conn_error) => {
		match conn_error {
		    ConnectError::NoWaylandLib => panic!("Connect Error: no wayland lib"),
		    ConnectError::InvalidFd => panic!("Connect Error: invalid fd"),
		    ConnectError::NoCompositor => panic!("Connect Error: no compositor"),
		}
	    }
	    Ok(conn) => conn
	};

	// No need for a round-trip, the display object exists implicitly with ID 1

	let mut event_queue = connection.new_event_queue();
	let display = connection.display();
	display.get_registry(&event_queue.handle(), ());

	println!("Parsing global objects");

	match event_queue.roundtrip(self) {
	    Err(_) => panic!("Parsing global objects nok"),
	    Ok(_) => println!("Parsing global objects ok"),
	}

	println!("Parsing global objects events");

	match event_queue.roundtrip(self) {
	    Err(_) => panic!("Parsing global objects events nok"),
	    Ok(_) => println!("Parsing global objects events ok"),
	}

	if self.supported_formats_vec.iter().any(|format| {
	    match format {
		wl_shm::Format::Argb8888 =>  return true,
		_ => return false,
	    }
	}) == false {
	    panic!("could not find Argb8888 in supported formats vec");
	}

	return event_queue;
    }

    pub fn main_loop(&mut self, event_queue: &mut EventQueue<WlApp>) {
	let mut next_timer: Option<Duration> = None;
	let bg_duration_as_duration = Duration::new(self.config.bg_duration_seconds, 0);
	let bg_duration_as_epoll_timer = match epoll::EpollTimeout::try_from(bg_duration_as_duration.as_millis()) {
            Ok(duration) => duration,
            Err(error) => panic!("Time between updates is incorrect: {}", error),
	};
	let acceptable_delta = Duration::new(1, 0);
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
		match event_queue.dispatch_pending(self) {
		    Ok(_) => (),
		    Err(error) => println!("dispatch event error : {}", error),
		}
	    }
	}}

	let now = Instant::now();
	next_timer = None;
	for (key,output) in self.output_map.iter_mut() {
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

		output.render(key, &event_queue.handle(), Some(&(self.image_list.get(index_in_image_array).unwrap().path)));
		output.next_redraw = Some(Instant::now() + bg_duration_as_duration);
		index_in_image_array = (index_in_image_array + 1) % self.image_list.len();
	    }
	}
	    }
    }
}
