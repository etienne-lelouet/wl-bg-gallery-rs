use std::{num::NonZeroUsize, os::fd::AsFd, time::Instant};
use wayland_client::{protocol::{wl_buffer, wl_output, wl_shm, wl_shm_pool, wl_surface}, QueueHandle};
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1;
use crate::{background_image::{fill_buffer_with_image, fill_buffer_random}, memory::MemoryMapping, wl_app::WlApp};

#[derive(Debug)]
pub struct Output {
    pub make: String,
    pub name: String,
    pub model: String,
    pub mode_height: u32,
    pub mode_width: u32,
    pub description: String,
    pub wl_output_proxy: Option<wl_output::WlOutput>,
    pub wl_surface_proxy: Option<wl_surface::WlSurface>,
    pub wlr_layer_surface_proxy: Option<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    pub wl_shm_pool: Option<wl_shm_pool::WlShmPool>,
    pub wl_buffer: Option<wl_buffer::WlBuffer>,
    pub mapping: Option<MemoryMapping>,
    pub serial_to_ack: u32,
    pub should_update_config: bool,
    pub next_redraw: Option<Instant>,
}

impl Output {
    pub fn new() -> Self {
	Self{
	    make: String::from(""),
	    model: String::from(""),
	    name: String::from(""),
	    mode_height: 0,
	    mode_width: 0,
	    description: String::from(""),
	    wl_output_proxy: None,
	    wl_surface_proxy: None,
	    wlr_layer_surface_proxy: None,
	    wl_shm_pool: None,
	    wl_buffer: None,
	    mapping: None,
	    serial_to_ack: 0,
	    should_update_config: true,
	    next_redraw: None
	}
    }

    pub fn get_memory_size(&self) -> NonZeroUsize {
	let shm_pool_size_usize = match usize::try_from(self.mode_width * self.mode_height * 4) {
	    Ok(shm_pool_size_usize) => shm_pool_size_usize,
	    Err(err) => panic!("failed to convert shm_pool_size to usize: {}", err)
	};
	match NonZeroUsize::new(shm_pool_size_usize) {
	    Some(shm_pool_size_nonzerousize) => shm_pool_size_nonzerousize,
	    None => panic!("failed to convert shm_pool_size to non zero usize")
	}
    }

    pub fn get_stride_i32(&self) -> i32 {
	match i32::try_from(self.mode_width * 4) {
	    Ok(stride) => stride,
	    Err(err) => panic!("failed to convert stride to i32: {}", err)
	}
    }

    pub fn get_memory_size_i32(&self) -> i32 {
	match i32::try_from(self.get_memory_size().get()) {
	    Ok(shm_pool_size_i32) => shm_pool_size_i32,
	    Err(err) => panic!("failed to convert shm_pool_size to i32: {}", err)
	}
    }

    pub fn configure_shm_pool(&mut self, key: &u32, wl_shm_proxy: &wl_shm::WlShm, qhandle: &QueueHandle<WlApp>) {
	self.mapping = match MemoryMapping::new(key.to_string(), self.get_memory_size()) {
	    Some(mapping) => Some(mapping),
	    None => panic!("Creating buffer failed !"),
	};

	self.wl_shm_pool =  Some(
	    wl_shm_proxy.create_pool(
		self.mapping.as_ref().unwrap().fd.as_fd(), self.get_memory_size_i32(), qhandle, *key
	    )
	);
    }

    pub fn render(&mut self, key: &u32, qhandle: &QueueHandle<WlApp>, to_draw: Option<String>) {
	let wl_shm_pool = self.wl_shm_pool.as_ref().unwrap();
	self.wl_buffer = Some(
	    wl_shm_pool.create_buffer(
		0,
		self.mode_width as i32,
		self.mode_height as i32,
		self.get_stride_i32(),
		wl_shm::Format::Argb8888,
		qhandle,
		*key
	    )
	);

	let buffer = self.wl_buffer.as_ref().unwrap();
	let ptr: & mut[u8];
	let mapping = self.mapping.as_ref().unwrap();
	unsafe {
	    ptr = std::slice::from_raw_parts_mut::<u8>(
		mapping.ptr.as_ptr() as *mut u8,
		mapping.size.get()
	    );
	}

	match to_draw {
	    Some(path) => fill_buffer_with_image(
		&path,
		self.mode_width,
		self.mode_height,
		ptr
	    ),
	    None => fill_buffer_random(ptr),
	};

	let surface = self.wl_surface_proxy.as_ref().unwrap();
	surface.set_buffer_scale(1);
	surface.attach(Some(buffer), 0, 0);
	surface.damage_buffer(0, 0, i32::MAX, i32::MAX);
	surface.commit();
    }

    pub fn clear(&mut self) {
	if let Some(ref layer_shell) = self.wlr_layer_surface_proxy {
	    layer_shell.destroy();
	    self.wlr_layer_surface_proxy = None;
	}
	if let Some(ref surface_proxy) = self.wl_surface_proxy {
	    surface_proxy.destroy();
	    self.wl_surface_proxy = None;
	}
	if let Some(ref wl_shm_pool) = self.wl_shm_pool {
	    wl_shm_pool.destroy();
	    self.wl_shm_pool = None;
	}
	if let Some(ref wl_buffer) = self.wl_buffer {
	    wl_buffer.destroy();
	    self.wl_buffer = None;
	}
	if let Some(ref mapping) = self.mapping {
	    if let Err(error) = mapping.destroy() {
		panic!("mapping destroy error {}", error);
	    }
	    self.mapping = None;
	}
	self.should_update_config = true;
	self.next_redraw = None;
    }
}
