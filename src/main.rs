#![doc = include_str!("../README.md")]

#[macro_use]
mod macros;

mod atoms;
mod errors;
mod events;
mod prelude;
mod wm;

use prelude::*;

/// Configure file logging.
///
/// This creates a new [`fern`] logger, with the [`LevelFilter`] set to
/// [`LevelFilter::Trace`] (when debug assertions are turned on) or else
/// [`LevelFilter::Info`], and writes to the path
/// `$XDG_CACHE_HOME/nerdwm/logs/nerdwm-{timestamp}.log`
fn setup_logger() {
    // TODO: propagate `Result`s, and some kind of fallback?

    let mut log_path = get_xdg_dirs().get_cache_home();
    log_path.push("logs");

    if !log_path.exists() {
        std::fs::create_dir_all(&log_path).unwrap();
    }

    // Log file with current timestamp.
    log_path.push(
        &format!(
            "{}.log",
            chrono::Local::now().format("nerdwm-%Y-%m-%d-%H:%M:%S")
        )[..],
    );

    #[cfg(debug_assertions)]
    let current_log_level = log::LevelFilter::Trace;

    #[cfg(not(debug_assertions))]
    let current_log_level = log::LevelFilter::Trace;

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ));
        })
        .level(current_log_level)
        .chain(fern::log_file(log_path).unwrap())
        .apply()
        .unwrap();
}

/// Set a new panic hook.
///
/// The new hook writes the panic message to stderr and logs it.
fn setup_panic() {
    std::panic::set_hook(Box::new(|info| {
        error!("{}", info);
        eprintln!("{}", info);
    }));
}

#[tokio::main]
async fn main() {
    setup_logger();
    setup_panic();

    let mut manager = wm::WindowManager::new().unwrap();
    manager.run().await.unwrap();
}
