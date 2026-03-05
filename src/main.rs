mod api;
mod app;
mod config;
mod messages;
mod theme;
mod views;

use bubbletea_rs::Program;
use tracing_subscriber::{fmt, EnvFilter};
use tracing_appender::rolling;

fn init_logging() {
    let file_appender = rolling::never(".", "heascale-tui.log");
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("debug"));

    fmt()
        .with_env_filter(filter)
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(false)
        .with_level(true)
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();
    tracing::info!("heascale-tui starting");

    let program = Program::<app::App>::builder()
        .alt_screen(true)
        .build()?;
    program.run().await?;

    tracing::info!("heascale-tui exiting");
    Ok(())
}
