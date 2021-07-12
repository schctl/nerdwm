#![allow(unused_imports)]

use log::{debug, info};

mod display_manager;
mod window;
mod wm;

use display_manager::DisplayManager;

/// Configure file logging.
fn setup_logger() {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("nerdwm").unwrap();

    let mut current_log = xdg_dirs.get_cache_home();
    current_log.push("logs");

    if !current_log.exists() {
        std::fs::create_dir_all(&current_log).unwrap();
    }

    // Log file with current timestamp.
    current_log.push(
        &format!(
            "{}.log",
            chrono::Local::now().format("nerdwm-%Y-%m-%d-%H:%M:%S")
        )[..],
    );

    #[cfg(debug_assertions)]
    let current_log_level = log::LevelFilter::Debug;

    #[cfg(not(debug_assertions))]
    let current_log_level = log::LevelFilter::Info;

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(current_log_level)
        .chain(fern::log_file(current_log).unwrap())
        .apply()
        .unwrap();
}

fn main() {
    setup_logger();

    let mut wm = wm::WindowManager::new();

    info!("Initialized.");

    wm.run()
}
