pub mod app;
pub mod config;
pub mod slicer;
pub mod utils;

// Re-export AppState for use in handlers
#[derive(Clone)]
pub struct AppState {
    pub config: config::Config,
}
