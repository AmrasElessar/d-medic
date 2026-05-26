use crate::paths;
use std::sync::OnceLock;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

static GUARD: OnceLock<WorkerGuard> = OnceLock::new();

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = paths::log_dir()?;
    std::fs::create_dir_all(&log_dir)?;

    let appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("dmedic")
        .filename_suffix("log")
        .max_log_files(7)
        .build(&log_dir)?;

    let (writer, guard) = tracing_appender::non_blocking(appender);
    let _ = GUARD.set(guard);

    // Varsayılan: genel info, kendi kodumuz debug, frontend köprüsü (dev_log
    // komutu target="frontend") debug. DMEDIC_LOG env ile override edilebilir.
    let filter = EnvFilter::try_from_env("DMEDIC_LOG")
        .unwrap_or_else(|_| EnvFilter::new("info,d_medic_lib=debug,frontend=debug"));

    let file_layer = fmt::layer()
        .with_writer(writer)
        .with_ansi(false)
        .with_target(true)
        .json();

    let registry = tracing_subscriber::registry().with(filter).with(file_layer);

    #[cfg(debug_assertions)]
    {
        let console_layer = fmt::layer().with_ansi(true).with_target(false);
        registry.with(console_layer).try_init()?;
    }

    #[cfg(not(debug_assertions))]
    {
        registry.try_init()?;
    }

    Ok(())
}
