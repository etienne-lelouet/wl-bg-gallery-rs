pub mod wl_app;
pub mod output;
pub mod memory;
pub mod background_image;
pub mod image_file;
pub mod config;
pub mod image_order_prio;
pub mod test;

use config::Config;
use wl_app::WlApp;

fn main() {
    let config = Config::get_config();
    let image_list = image_file::get_image_list(&config.path, &config.authorized_formats);
    if image_list.len() == 0 {
	panic!("No images to set as background !");
    }

    let mut wl_app = WlApp::new(config, image_list);
    wl_app.run();
}
