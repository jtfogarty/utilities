use crate::cli::Cli;
use anyhow::Result;
use clap::Parser;

mod cli;
mod config;
mod x;
mod server;
mod tools;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    match cli.command {
        cli::Commands::Start { config } => {
            server::start_server(config).await?;
        }
    }
    Ok(())
}