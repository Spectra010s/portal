use {
    crate::config::models::PortalConfig,
    chrono::Local,
    std::{env, fs::create_dir_all},
    tracing::{debug, trace},
    tracing_appender::non_blocking::WorkerGuard,
    tracing_subscriber::{filter::LevelFilter, fmt, prelude::*, EnvFilter},
};

/// Initialize the global logger
pub async fn init(verbose: bool, quiet: bool) -> WorkerGuard {
    // Ensure ~/.portal/_logs/ exists
    let home_dir = PortalConfig::get_dir()
        .await
        .expect("Could not determine log directory");
    let log_dir = home_dir.join("_logs");
    trace!("Checking/Creating log directory: {:?}", log_dir);
    create_dir_all(&log_dir).expect("Failed to create log directory");

    let ts = Local::now().format("%Y%m%d-%H%M%S-%3f");
    let log_name = format!("portal-{}.log", ts);
    let file_appender = tracing_appender::rolling::never(&log_dir, &log_name);
    trace!("Rolling file appender configured for {:?}", log_dir);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let env_directive = env::var("PORTAL_LOG")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| env::var("RUST_LOG").ok().filter(|v| !v.trim().is_empty()));
    let file_filter = env_directive
        .as_deref()
        .and_then(|raw| EnvFilter::try_new(raw).ok())
        .unwrap_or_else(|| EnvFilter::new("debug"));

    let terminal_level = if quiet {
        LevelFilter::ERROR
    } else if verbose {
        LevelFilter::INFO
    } else {
        LevelFilter::WARN
    };

    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(std::io::stderr)
                .without_time()
                .with_target(false)
                .with_filter(terminal_level),
        )
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_target(false)
                .with_filter(file_filter),
        )
        .init();

    debug!("Global tracing subscriber initialized");
    tracing::info!("Logger initialized in {:?}", home_dir);
    guard
}
