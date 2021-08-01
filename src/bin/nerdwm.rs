use log::*;

use nerdwm::config::Config;
use nerdwm::wm::WindowManager;

/// Configure file logging.
fn setup_logger() {
    // ~/.cache/nerdwm
    let xdg_dirs = xdg::BaseDirectories::with_prefix("nerdwm").unwrap();

    let mut log_path = xdg_dirs.get_cache_home();
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
        .chain(fern::log_file(log_path).unwrap())
        .apply()
        .unwrap();
}

fn main() {
    // ~/.config/nerdwm
    let xdg_dirs = xdg::BaseDirectories::with_prefix("nerdwm").unwrap();

    setup_logger();

    // Read configuration file
    let config = {
        let mut config = xdg_dirs.get_config_home();
        if !config.exists() {
            std::fs::create_dir_all(&config).unwrap();
        }
        config.push("config.json");

        Config::new(&config)
    };

    let mut wm = WindowManager::new(config);
    info!("Initialized.");
    wm.run()
}
