use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "saltapimcp")]
#[command(about = "SaltStack MCP server for LLM agents")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the MCP server (stdio transport)
    Start {
        #[command(flatten)]
        config: crate::config::ServerConfig,
    },
}