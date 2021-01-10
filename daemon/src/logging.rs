#[cfg(debug_assertions)]
const DEFAULT_LOGGING_LEVEL: log::LevelFilter = log::LevelFilter::Debug;
#[cfg(not(debug_assertions))]
const DEFAULT_LOGGING_LEVEL: log::LevelFilter = log::LevelFilter::Info;

use log::{Level, Metadata, Record};

use crate::{Error, Result};

pub struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

/// begins loggin with fern, in a nicely formatted way
#[cfg(feature = "pretty_log")]
pub fn init_logging() -> Result<()> {
    if cfg!(feature = "pretty_log") {
        use chrono::Local;
        fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "[{timestamp}][{level}][{target}] {message}",
                    level = record.level(),
                    timestamp = Local::now().format("%Y-%m-%dT%H:%M:%S%.3f"),
                    target = record.target(),
                    message = message,
                ))
            })
            .chain(std::io::stdout())
            .level(DEFAULT_LOGGING_LEVEL)
            .level_for("mio", log::LevelFilter::Info)
            .level_for("tokio_core", log::LevelFilter::Info)
            // .chain(fern::log_file("output.log")?)
            .apply()?;
        Ok(())
    } else {
        log::set_logger(&SimpleLogger)
            .map(|()| log::set_max_level(DEFAULT_LOGGING_LEVEL))
            .map_err(Error::SetLoggerError)
    }
}
