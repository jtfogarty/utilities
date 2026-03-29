use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "xapimcp")]
#[command(about = "X.com (Twitter) MCP server for LLM agents – personal bookmarks")]
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