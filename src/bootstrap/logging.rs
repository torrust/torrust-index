//! Setup for the application logging.
//!
//! - `Off`
//! - `Error`
//! - `Warn`
//! - `Info`
//! - `Debug`
//! - `Trace`
use std::sync::Once;

use tracing::info;
use tracing::level_filters::LevelFilter;

use crate::config::Threshold;

static INIT: Once = Once::new();

pub fn setup(threshold: &Threshold) {
    let tracing_level_filter: LevelFilter = threshold.clone().into();

    if tracing_level_filter == LevelFilter::OFF {
        return;
    }

    INIT.call_once(|| {
        tracing_stdout_init(tracing_level_filter, &TraceStyle::Default);
    });
}

fn tracing_stdout_init(filter: LevelFilter, style: &TraceStyle) {
    let builder = tracing_subscriber::fmt().with_max_level(filter);

    let () = match style {
        TraceStyle::Default => builder.init(),
        TraceStyle::Pretty(display_filename) => builder.pretty().with_file(*display_filename).init(),
        TraceStyle::Compact => builder.compact().init(),
        TraceStyle::Json => builder.json().init(),
    };

    info!("Logging initialized");
}

#[derive(Debug)]
pub enum TraceStyle {
    Default,
    Pretty(bool),
    Compact,
    Json,
}

impl std::fmt::Display for TraceStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let style = match self {
            TraceStyle::Default => "Default Style",
            TraceStyle::Pretty(path) => match path {
                true => "Pretty Style with File Paths",
                false => "Pretty Style without File Paths",
            },

            TraceStyle::Compact => "Compact Style",
            TraceStyle::Json => "Json Format",
        };

        f.write_str(style)
    }
}
