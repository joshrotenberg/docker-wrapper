//! Redis Sentinel instance management

use anyhow::Result;
use colored::*;

use crate::cli::SentinelAction;

pub async fn handle_action(action: SentinelAction, _verbose: bool) -> Result<()> {
    match action {
        SentinelAction::Start(_args) => {
            println!(
                "{} Redis Sentinel support is not yet implemented",
                "Warning:".yellow()
            );
            println!("This command will set up Redis Sentinel for high availability monitoring");
            Ok(())
        }
        SentinelAction::Stop(_args) => {
            println!(
                "{} Redis Sentinel support is not yet implemented",
                "Warning:".yellow()
            );
            Ok(())
        }
        SentinelAction::Info(_args) => {
            println!(
                "{} Redis Sentinel support is not yet implemented",
                "Warning:".yellow()
            );
            Ok(())
        }
    }
}
