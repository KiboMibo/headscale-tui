mod api;
mod app;
mod config;
mod messages;
mod theme;
mod views;

use bubbletea_rs::Program;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let program = Program::<app::App>::builder()
        .alt_screen(true)
        .build()?;
    program.run().await?;
    Ok(())
}
