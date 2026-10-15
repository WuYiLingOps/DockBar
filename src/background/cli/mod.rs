mod debugger;
mod self_pipe;
pub mod shortcuts;
mod uri;

pub use self_pipe::SelfPipe;

use std::sync::atomic::Ordering;

use dockbar_ipc::{AppIpc, commands::AppCommand, messages::AppMessage};

use crate::{
    error::Result,
    resources::cli as resources_cli,
    widgets::{cli as widget_cli, dock::cli as dock_cli, popups::cli as popups_cli, show_settings},
};

/// WARNING: dockbar.exe CLI commands are deprecated.
///
/// Use `dockbar-cli` instead.
#[derive(Debug, clap::Parser)]
#[command(version, name = "DockBar")]
struct MainCli {
    #[arg(long, default_value_t)]
    silent: bool,
    #[arg(long, default_value_t)]
    verbose: bool,
    /// Path or URI to open (e.g. from the Windows protocol handler).
    uri: Option<String>,
}

/// Called at startup by the main executable. If a URI is present it is forwarded
/// to the running instance via IPC and the process exits; otherwise returns Ok(())
/// so normal app startup continues.
pub async fn handle_console_client() -> Result<()> {
    use clap::Parser;
    let cli = match MainCli::try_parse() {
        Ok(cli) => cli,
        Err(e) => e.exit(),
    };

    if cli.silent {
        crate::SILENT.store(true, Ordering::SeqCst);
    }
    if cli.verbose {
        crate::VERBOSE.store(true, Ordering::SeqCst);
        println!("Received args: {:#?}", std::env::args().collect::<Vec<_>>());
        println!("Parsed CLI: {cli:#?}");
    }

    if let Some(uri) = cli.uri {
        AppIpc::send(AppMessage::OpenUri(uri))
            .await
            .map_err(|_| "Can't establish connection, ensure DockBar is running.")?;
        std::process::exit(0);
    }

    Ok(())
}

pub async fn process_app_command(cmd: AppCommand) -> Result<()> {
    match cmd {
        AppCommand::Settings => {
            show_settings()?;
        }
        AppCommand::Debugger(command) => {
            debugger::process(command)?;
        }
        AppCommand::Dock(command) => {
            dock_cli::process(command)?;
        }
        AppCommand::Widget(command) => {
            widget_cli::run(command)?;
        }
        AppCommand::Resource(command) => {
            resources_cli::process(command).await?;
        }
        AppCommand::Popup(command) => {
            popups_cli::process(command)?;
        }
        AppCommand::ToggleShortcutsPause => {
            shortcuts::toggle_pause();
        }
        _ => {
            return Err("Command does not support instance execution".into());
        }
    }
    Ok(())
}
