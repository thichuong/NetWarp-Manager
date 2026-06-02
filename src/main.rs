#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

mod callbacks;
pub mod error;
mod helpers;
mod net_utils;
mod polling;
mod warp;
mod wifi;

pub use error::AppError;

use slint::ComponentHandle;
use std::rc::Rc;

slint::include_modules!();

#[tokio::main]
async fn main() -> Result<(), slint::PlatformError> {
    // 1. Initialize the main Slint UI Application Window
    let ui = AppWindow::new()?;

    // 2. Setup Vector Models to handle dynamic arrays on Slint UI
    let wifi_list_model = Rc::new(slint::VecModel::<WifiNetwork>::default());
    ui.set_wifi_list(wifi_list_model.into());

    let download_history_model = Rc::new(slint::VecModel::<f32>::from(vec![0.0; 25]));
    ui.set_download_history(download_history_model.into());

    let upload_history_model = Rc::new(slint::VecModel::<f32>::from(vec![0.0; 25]));
    ui.set_upload_history(upload_history_model.into());

    ui.set_max_history_value(100.0);
    ui.set_max_history_label("100 KB/s".into());

    // 3. Register UI callbacks
    callbacks::register_callbacks(&ui);

    // 4. Start background polling loops
    polling::start_polling_loops(&ui);

    // 5. Run the Slint Event Loop (This blocks until the window is closed)
    ui.run()?;
    Ok(())
}
