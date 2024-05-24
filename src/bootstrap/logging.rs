//! Setup for the application logging.
//!
//! - `Off`
//! - `Error`
//! - `Warn`
//! - `Info`
//! - `Debug`
//! - `Trace`
use std::sync::Once;

use log::{info, LevelFilter};

use crate::config::v1::LogLevel;

static INIT: Once = Once::new();

pub fn setup(log_level: &Option<LogLevel>) {
    let level = config_level_or_default(log_level);

    if level == log::LevelFilter::Off {
        return;
    }

    INIT.call_once(|| {
        stdout_config(level);
    });
}

fn config_level_or_default(log_level: &Option<LogLevel>) -> LevelFilter {
    match log_level {
        None => log::LevelFilter::Info,
        Some(level) => match level {
            LogLevel::Off => LevelFilter::Off,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        },
    }
}

fn stdout_config(level: LevelFilter) {
    if let Err(_err) = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}][{}] {}",
                chrono::Local::now().format("%+"),
                record.target(),
                record.level(),
                message
            ));
        })
        .level(level)
        .chain(std::io::stdout())
        .apply()
    {
        panic!("Failed to initialize logging.")
    }

    info!("logging initialized.");
}
