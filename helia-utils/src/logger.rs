//! Logger implementations

use crate::LoggerConfig;
use helia_interface::ComponentLogger;

/// Tracing-based logger implementation
pub struct TracingLogger {
    _config: LoggerConfig,
}

impl TracingLogger {
    pub fn new(config: LoggerConfig) -> Self {
        // Initialize tracing subscriber if not already initialized
        let _ = tracing_subscriber::fmt()
            .with_max_level(config.level)
            .with_ansi(true)
            .try_init();

        Self { _config: config }
    }
}

impl ComponentLogger for TracingLogger {
    fn debug(&self, message: &str) {
        tracing::debug!("{}", message);
    }

    fn info(&self, message: &str) {
        tracing::info!("{}", message);
    }

    fn warn(&self, message: &str) {
        tracing::warn!("{}", message);
    }

    fn error(&self, message: &str) {
        tracing::error!("{}", message);
    }
}
