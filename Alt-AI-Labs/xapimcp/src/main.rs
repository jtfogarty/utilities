use crate::cli::Cli;
use anyhow::Result;
use clap::Parser;

mod dotenv_load;
mod cli;
mod config;
mod x;
mod x_oauth;
mod server;
mod tools;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv_load::load();

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