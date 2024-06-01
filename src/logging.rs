#[cfg(unix)]
use std::os::unix::prelude::*;
#[cfg(windows)]
use std::os::windows::prelude::*;
use std::{fs::File, io, path::Path};

use eyre::Result;
use time::{macros::format_description, UtcOffset};
use tracing_subscriber::{
    fmt::{layer, time::OffsetTime},
    prelude::*,
    registry,
};

/// initialize logging to stdout and a log file
pub fn init_logging(log_path: &Path) -> Result<()> {
    let ts_format = format_description!("[day]-[month] [hour]:[minute]");
    let ts_offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
    let timer = OffsetTime::new(ts_offset, ts_format);

    let file = {
        let mut opts = File::options();
        opts.write(true).create(true).truncate(true);
        #[cfg(unix)]
        opts.custom_flags(libc::O_SYNC);
        #[cfg(windows)]
        opts.custom_flags(
            windows::Win32::Storage::FileSystem::FILE_FLAG_WRITE_THROUGH.0,
        );
        opts.open(log_path)?
    };

    let stdout_layer = layer()
        .with_writer(io::stdout)
        .with_target(false)
        .with_level(false)
        .with_timer(timer.clone());
    let file_layer = layer()
        .with_writer(file)
        .with_target(false)
        .with_level(false)
        .with_timer(timer)
        .with_ansi(false);

    registry().with(stdout_layer).with(file_layer).try_init()?;
    Ok(())
}
