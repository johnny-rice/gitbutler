use anyhow::{Context, Ok, Result};

mod args;
use args::{Args, Subcommands, actions, claude};
use but_settings::AppSettings;
mod command;
mod id;
mod log;
mod mcp;
mod mcp_internal;
mod metrics;
mod rub;
mod status;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = clap::Parser::parse();
    let app_settings = AppSettings::load_from_default_path_creating()?;

    let namespace = option_env!("IDENTIFIER").unwrap_or("com.gitbutler.app");
    gitbutler_secret::secret::set_application_namespace(namespace);

    match &args.cmd {
        Subcommands::Mcp { internal } => {
            if *internal {
                mcp_internal::start().await
            } else {
                mcp::start(app_settings).await
            }
        }
        Subcommands::Actions(actions::Platform { cmd }) => match cmd {
            Some(actions::Subcommands::HandleChanges {
                description,
                handler,
            }) => {
                let handler = *handler;
                command::handle_changes(&args.current_dir, args.json, handler, description)
            }
            None => command::list_actions(&args.current_dir, args.json, 0, 10),
        },
        Subcommands::Claude(claude::Platform { cmd }) => match cmd {
            claude::Subcommands::PreTool => {
                let out = command::claude::handle_pre_tool_call()?;
                println!("{}", serde_json::to_string(&out)?);
                Ok(())
            }
            claude::Subcommands::PostTool => {
                let out = command::claude::handle_post_tool_call()?;
                println!("{}", serde_json::to_string(&out)?);
                Ok(())
            }
            claude::Subcommands::Stop => {
                let out = command::claude::handle_stop().await?;
                println!("{}", serde_json::to_string(&out)?);
                Ok(())
            }
        },
        Subcommands::Log => log::commit_graph(&args.current_dir, args.json),
        Subcommands::Status => status::worktree(&args.current_dir, args.json),
        Subcommands::Rub { source, target } => {
            let result = rub::handle(&args.current_dir, args.json, source, target)
                .context("Rubbed the wrong way.");
            if let Err(e) = &result {
                eprintln!("{} {}", e, e.root_cause());
            }
            Ok(())
        }
    }
}
