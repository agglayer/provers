use log::Log;
use serde::{Deserialize, Serialize};
use tracing_subscriber::{prelude::*, EnvFilter};

pub mod log;

/// The log format.
#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    #[default]
    Pretty,
    Json,
}
pub fn tracing(config: &Log) {
    // TODO: Support multiple outputs.
    let writer = config.outputs.first().cloned().unwrap_or_default();

    let layer = match config.format {
        LogFormat::Pretty => tracing_subscriber::fmt::layer()
            .pretty()
            .with_writer(writer.as_make_writer())
            .with_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| config.level.into()))
            .boxed(),

        LogFormat::Json => tracing_subscriber::fmt::layer()
            .json()
            .with_writer(writer.as_make_writer())
            .with_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| config.level.into()))
            .boxed(),
    };

    tracing_subscriber::Registry::default().with(layer).init();
}
