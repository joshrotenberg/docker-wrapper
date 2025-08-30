//! Redis Developer CLI Tool
//!
//! A command-line tool for quickly spinning up Redis development environments
//! including basic Redis, Redis Stack, Redis Cluster, Redis Sentinel, and Redis Enterprise.

use anyhow::Result;
use clap::Parser;
use colored::*;

mod cli;
mod commands;
mod config;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let env_filter = if cli.verbose {
        "redis_dev=debug"
    } else {
        "redis_dev=info"
    };
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .init();

    // Initialize configuration
    config::ensure_config_dir()?;

    match cli.command {
        Some(Commands::Basic { action }) => {
            commands::basic::handle_action(action, cli.verbose).await?;
        }
        Some(Commands::Stack { action }) => {
            commands::stack::handle_action(action, cli.verbose).await?;
        }
        Some(Commands::Cluster { action }) => {
            commands::cluster::handle_action(action, cli.verbose).await?;
        }
        Some(Commands::Sentinel { action }) => {
            commands::sentinel::handle_action(action, cli.verbose).await?;
        }
        Some(Commands::Enterprise { action }) => {
            commands::enterprise::handle_action(action, cli.verbose).await?;
        }
        Some(Commands::List { r#type }) => {
            commands::list::handle_list(r#type, cli.verbose).await?;
        }
        Some(Commands::Cleanup { force, r#type }) => {
            commands::cleanup::handle_cleanup(force, r#type, cli.verbose).await?;
        }
        Some(Commands::Logs {
            name,
            follow,
            tail,
            timestamps,
        }) => {
            commands::logs::handle_logs(name, follow, tail, timestamps, cli.verbose).await?;
        }
        None => {
            println!("{}", "Redis Developer Tool".bold().cyan());
            println!();
            println!("Quick commands to get started:");
            println!("  {} Start basic Redis", "redis-dev basic start".green());
            println!(
                "  {} Start Redis + shell",
                "redis-dev basic start --shell".green()
            );
            println!(
                "  {} Start Redis Stack with popular modules",
                "redis-dev stack start".green()
            );
            println!(
                "  {} Start 3-node Redis Cluster",
                "redis-dev cluster start --masters 3".green()
            );
            println!(
                "  {} Start Redis Enterprise cluster",
                "redis-dev enterprise start --nodes 3".green()
            );
            println!();
            println!("  {} List all running instances", "redis-dev list".yellow());
            println!(
                "  {} View logs for instances",
                "redis-dev logs --follow".blue()
            );
            println!("  {} Clean up all instances", "redis-dev cleanup".red());
            println!();
            println!(
                "Use {} for detailed help on any command.",
                "--help".dimmed()
            );
        }
    }

    Ok(())
}
