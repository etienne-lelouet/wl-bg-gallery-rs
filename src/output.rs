use wayland_client::protocol::{wl_buffer, wl_output, wl_shm_pool, wl_surface};
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1;
use crate::memory::MemoryMapping;

#[derive(Debug)]
pub struct Output {
    pub make: String,
    pub model: String,
    pub mode_height: i32,
    pub mode_width: i32,
    pub description: String,
    pub wl_output_proxy: Option<wl_output::WlOutput>,
    pub wl_surface_proxy: Option<wl_surface::WlSurface>,
    pub wlr_layer_surface_proxy: Option<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    pub wl_shm_pool: Option<wl_shm_pool::WlShmPool>,
    pub wl_buffer: Option<wl_buffer::WlBuffer>,
    pub mapping: Option<MemoryMapping>
}

impl Output {
    pub fn new() -> Self {
	Self{
	    make: String::from(""),
	    model: String::from(""),
	    mode_height: 0,
	    mode_width: 0,
	    description: String::from(""),
	    wl_output_proxy: None,
	    wl_surface_proxy: None,
	    wlr_layer_surface_proxy: None,
	    wl_shm_pool: None,
	    wl_buffer: None,
	    mapping: None
	}
    }
}
