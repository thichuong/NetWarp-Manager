#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod cache;
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

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::needless_borrow,
    clippy::clone_on_copy,
    clippy::shadow_unrelated
)]
mod ui_generated {
    slint::include_modules!();
}
pub use ui_generated::*;

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<(), slint::PlatformError> {
    // 1. Initialize the main Slint UI Application Window
    let ui = AppWindow::new()?;

    // Set the detected OS Platform name for UI display
    ui.set_os_platform(helpers::detect_os_name().into());

    // 2. Setup Vector Models to handle dynamic arrays on Slint UI
    let wifi_list_model = Rc::new(slint::VecModel::<WifiNetwork>::default());
    ui.set_wifi_list(wifi_list_model.into());

    let download_history_model = Rc::new(slint::VecModel::<f32>::from(vec![0.0; 25]));
    ui.set_download_history(download_history_model.clone().into());

    let upload_history_model = Rc::new(slint::VecModel::<f32>::from(vec![0.0; 25]));
    ui.set_upload_history(upload_history_model.clone().into());

    helpers::init_history_models(download_history_model, upload_history_model);

    let console_logs_model = Rc::new(slint::VecModel::<slint::SharedString>::from(vec![
        "[System] Engine initialized. Awaiting user input commands.".into(),
    ]));
    ui.set_console_logs(console_logs_model.clone().into());
    helpers::init_logs_model(console_logs_model);

    // 3. Register UI callbacks
    callbacks::register_callbacks(&ui);

    // 4. Start background polling loops
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    polling::start_polling_loops(&ui, shutdown_rx);

    // 6. Run the Slint Event Loop (This blocks until the window is closed)
    let run_result = ui.run();

    // Save UI states to local JSON cache on exit
    cache::save_cache_from_ui(&ui);

    // Send shutdown signal to terminate background worker tasks
    let _ = shutdown_tx.send(true);

    run_result?;
    Ok(())
}
