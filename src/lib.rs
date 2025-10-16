pub mod core;

use lazy_static::lazy_static;
use std::env;
lazy_static! {
    // Define the directory where images are stored, defaulting to "$HOME/Pictures" if not set
    pub static ref IMAGE_DIR: String =
        env::var("IMAGE_DIR").unwrap_or_else(|_| format!("{}/Pictures", env::var("HOME").unwrap()));

    // Initialize a global instance of ImageCache using the specified image directory
    pub static ref IC: core::image_cache::PhotoCache =
        core::image_cache::PhotoCache::build(IMAGE_DIR.as_str()).unwrap();
}
pub mod handler;
pub mod resources;
pub mod server;
pub mod tools;
