use {
    crate::config::models::PortalConfig,
    std::fs::create_dir_all,
    tracing_appender::non_blocking::WorkerGuard,
    tracing_subscriber::{EnvFilter, fmt, prelude::*},
};

/// Initialize the global logger
pub async fn init() -> WorkerGuard {
    // Ensure ~/.portal/_logs/ exists
    let home_dir = PortalConfig::get_dir()
        .await
        .expect("Could not determine log directory");
    let log_dir = home_dir.join("_logs");
    create_dir_all(&log_dir).expect("Failed to create log directory");

    let file_appender = tracing_appender::rolling::daily(log_dir, "portal.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_target(false),
        )
        .init();

    tracing::info!("Logger initialized in {:?}", home_dir);
    guard
}
