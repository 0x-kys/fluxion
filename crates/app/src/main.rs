use clap::Parser;
use fluxion_core::Editor;
use fluxion_tui::Tui;
use std::error::Error;
use tracing::{Level, info};

/// Fluxion: A Rust-based text editor
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let args = Args::parse();
    info!("Starting Fluxion with args: {:?}", args);

    let mut editor = Editor::new("Hello World from Fluxion!");

    let mut tui = Tui::new()?;
    tui.run(&mut editor)?;

    Ok(())
}
