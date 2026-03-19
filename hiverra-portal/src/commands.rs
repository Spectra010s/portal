use {
    crate::{
        config::{
            list::list_config, set::set_config, setup::handle_setup, show::show_config_value,
        },
        history::{
            build_history_json_detail_list, build_history_json_list, clear_history,
            delete_history_record, filter_history, format_history_detail, load_history,
            output_history_json_detail, output_history_json_list, output_history_table,
            parse_since_unix, HistoryMode,
        },
        receiver::start_receiver,
        sender::start_send,
        update::update_portal,
    },
    anyhow::{Context, Result},
    clap::{Args, Subcommand},
    std::path::PathBuf,
    tokio::fs,
    tracing::{debug, info, trace, warn},
};

// 2. Defining the Choices (The Enum)
#[derive(Subcommand)]
pub enum Commands {
    /// Send a file
    Send {
        /// The files or folders to send. If empty, opens the interactive picker.
        file: Option<Vec<PathBuf>>,
        /// The IP address of the receiver
        #[arg(short, long)]
        address: Option<String>,
        /// The port the receiver is listening on
        #[arg(short, long, default_value_t = 7878)]
        port: u16,
        /// The username of the receiver
        /// If omitted, Portal will prompt you for a name.
        #[arg(short, long, value_name = "USERNAME")]
        to: Option<String>,
        /// Send folder recursively
        #[arg(short, long, value_name = "FOLDER")]
        recursive: bool,
        /// Disable gzip compression for tranfer
        #[arg(long)]
        no_compress: bool,
    },
    /// Receive a file
    Receive {
        /// Specify which port to use
        #[arg(short, long)]
        port: Option<u16>,
        /// Directory where received files will be saved
        #[arg(short, long, value_name = "PATH")]
        dir: Option<PathBuf>,
    },
    /// Update portal to latest version
    Update,
    /// Show transfer history and manage saved records
    History {
        /// Action to perform on history records
        #[command(subcommand)]
        action: Option<HistoryAction>,
        /// Show a specific record 
        id: Option<usize>,
        /// Show all items in detail view
        #[arg(long)]
        items_all: bool,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
        /// Filter history results
        #[command(flatten)]
        filter: HistoryFilterArgs,
    },
    /// Configuration Settings management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Set or Update a setting
    Set {
        /// The configuration key to change
        key: String,
        /// The new value for the setting
        value: String,
    },
    /// View current settings variable value
    Show { key: String },
    /// List all cureent configuration settings
    List,
    /// Initialize or reconfigure Portal's default settings interactively
    Setup,
}

#[derive(Subcommand, Debug)]
pub enum HistoryAction {
    /// Clear all history records
    Clear,
    /// Delete a history record by ID (newest-first)
    Delete {
        /// The record ID shown in `portal history` output
        id: usize,
    },
    /// Export history to a file
    Export {
        /// Path to write the exported history file
        #[arg(short, long, value_name = "PATH")]
        output: Option<PathBuf>,
        /// Export file format (currently JSON only)
        #[arg(long, value_name = "json", default_value = "json", value_parser = ["json"])]
        format: String,
        /// Export all records (ignores limit)
        #[arg(long)]
        all: bool,
        /// Export detailed records (includes items and totals)
        #[arg(long)]
        detailed: bool,
        /// Filter which records get exported
        #[command(flatten)]
        filter: HistoryFilterArgs,
    },
}

#[derive(Args, Debug, Clone)]
pub struct HistoryFilterArgs {
    /// Limit number of records (default: 10)
    #[arg(short = 'n', long)]
    pub limit: Option<u32>,
    /// Filter by mode
    #[arg(short, long, value_name = "send|receive", value_parser = ["send", "receive"])]
    pub mode: Option<String>,
    /// Filter by date (e.g., 2026-03-16)
    #[arg(short, long, value_name = "YYYY-MM-DD")]
    pub since: Option<String>,
}

fn resolve_history_filters(
    parent: &HistoryFilterArgs,
    child: Option<&HistoryFilterArgs>,
) -> Result<(Option<HistoryMode>, Option<u64>, usize)> {
    trace!(
        "Resolving history filters: parent={:?}, child={:?}",
        parent,
        child
    );
    let mode_raw = child
        .and_then(|c| c.mode.clone())
        .or_else(|| parent.mode.clone());
    let since_raw = child
        .and_then(|c| c.since.clone())
        .or_else(|| parent.since.clone());
    let limit_raw = child
        .and_then(|c| c.limit)
        .or(parent.limit)
        .unwrap_or(10);

    let mode = match mode_raw.as_deref() {
        Some("send") => Some(HistoryMode::Send),
        Some("receive") => Some(HistoryMode::Receive),
        _ => None,
    };
    let since = match since_raw.as_deref() {
        Some(value) => Some(parse_since_unix(value)?),
        None => None,
    };
    trace!(
        "Resolved history filters: mode={:?}, since={:?}, limit={}",
        mode,
        since,
        limit_raw
    );

    Ok((mode, since, limit_raw as usize))
}

impl Commands {
    // This is the method attached to the Enum
    // return Result<()> to catch errors from sender/receiver
    pub async fn execute(&self) -> Result<()> {
        match self {
            Commands::Send {
                file,
                address,
                port,
                to,
                recursive,
                no_compress,
            } => {
                info!("Command: SEND initiated");
                debug!(
                    "Params: file={:?}, address={:?}, port={}, to={:?}, recursive={}, no_compress={}",
                    file, address, port, to, recursive, no_compress
                );
                trace!("Delegating to sender::start_send()");
                // send file or files
                start_send(&file, &address, &port, &to, &recursive, &no_compress)
                    .await
                    .context("Failed to execute Send command")?;
                trace!("sender::start_send() completed successfully");
            }
            Commands::Receive { port, dir } => {
                info!("Command: RECEIVE initiated");
                debug!("Params: port={:?}, dir={:?}", port, dir);
                trace!("Delegating to receiver::start_receiver()");
                // Pass the error up if receiving fails
                start_receiver(*port, &dir)
                    .await
                    .context("Failed to execute Receive command")?;
                trace!("receiver::start_receiver() completed successfully");
            }
            Commands::Update => {
                info!("Command: UPDATE initiated");
                trace!("Delegating to update::update_portal()");
                update_portal()
                    .await
                    .context("Failed to execute Update commamd")?;
                trace!("update::update_portal() completed successfully");
            }
            Commands::History {
                action,
                id,
                items_all,
                json,
                filter,
            } => {
                info!("Command: HISTORY initiated");
                debug!(
                    "Params: action={:?}, id={:?}, items_all={}, json={}, filter={:?}",
                    action, id, items_all, json, filter
                );
                let (filter_mode, since_unix, list_limit) =
                    resolve_history_filters(filter, None)?;
                if let Some(action) = action {
                    trace!("History action requested: {:?}", action);
                    match action {
                        HistoryAction::Clear => {
                            info!("History action: CLEAR");
                            trace!("Delegating to history::clear_history()");
                            clear_history().await?;
                            trace!("history::clear_history() completed successfully");
                            println!("Portal: History cleared.");
                            return Ok(());
                        }
                        HistoryAction::Delete { id } => {
                            info!("History action: DELETE id={}", id);
                            trace!("Delegating to history::delete_history_record()");
                            if delete_history_record(*id).await? {
                                trace!("history::delete_history_record() completed successfully");
                                println!("Portal: History record #{} deleted.", id);
                            } else {
                                warn!("User provided invalid history id: {}", id);
                                println!("Portal: Invalid history id {}", id);
                            }
                            return Ok(());
                        }
                        HistoryAction::Export {
                            output,
                            format,
                            all,
                            detailed,
                            filter: export_filter,
                        } => {
                            info!("History action: EXPORT format={}", format);
                            trace!("Delegating to history::load_history()");
                            let records = load_history().await?;
                            let (export_mode, export_since, export_limit) =
                                resolve_history_filters(filter, Some(export_filter))?;
                            let export_limit = if *all { 0 } else { export_limit };
                            trace!("Delegating to history::filter_history()");
                            let records = filter_history(
                                records,
                                export_mode,
                                export_since,
                                export_limit,
                            );
                            let json = if *detailed {
                                trace!("Delegating to history::build_history_json_detail_list()");
                                build_history_json_detail_list(records)?
                            } else {
                                trace!("Delegating to history::build_history_json_list()");
                                build_history_json_list(records)?
                            };
                            let out_path = output
                                .clone()
                                .unwrap_or_else(|| PathBuf::from("portal_history.json"));
                            trace!("Delegating to tokio::fs::write()");
                            fs::write(&out_path, json).await?;
                            trace!("history export completed successfully");
                            println!("Portal: History exported to {}", out_path.display());
                            return Ok(());
                        }
                    }
                }
                trace!("Delegating to history::load_history()");
                let records = load_history().await?;
                trace!("history::load_history() completed successfully");

                trace!("Delegating to history::filter_history()");
                let mut records = filter_history(records, filter_mode, since_unix, list_limit);
                trace!("history::filter_history() completed successfully");

                if records.is_empty() {
                    info!("History query returned no records.");
                    println!("Portal: No history records found.");
                    return Ok(());
                }

                if let Some(id) = *id {
                    if id == 0 || id > records.len() {
                        warn!("User provided invalid history id: {}", id);
                        println!("Portal: Invalid history id {}", id);
                        return Ok(());
                    }
                    let record = records.remove(id - 1);
                    if *json {
                        trace!("Delegating to history::output_history_json_detail()");
                        output_history_json_detail(&record, id)?;
                        trace!("history::output_history_json_detail() completed successfully");
                        return Ok(());
                    }
                    trace!("Delegating to history::format_history_detail()");
                    for line in format_history_detail(&record, id, *items_all) {
                        println!("{}", line);
                    }
                    trace!("history::format_history_detail() completed successfully");
                    return Ok(());
                }

                if *json {
                    trace!("Delegating to history::output_history_json_list()");
                    output_history_json_list(records)?;
                    trace!("history::output_history_json_list() completed successfully");
                    return Ok(());
                }

                trace!("Delegating to history::output_history_table()");
                output_history_table(&records);
                trace!("history::output_history_table() completed successfully");
            }
            Commands::Config { action } => {
                debug!("Config action: {:?}", action);
                match action {
                    ConfigAction::Set { key, value } => {
                        info!("Config: SET key='{}'", key);
                        trace!("Delegating to config::set::set_config");
                        set_config(&key, &value)
                            .await
                            .context("Failed to set configuration")?;
                    }
                    ConfigAction::Show { key } => {
                        info!("Config: SHOW key='{}'", key);
                        trace!("Delegating to config::show::show_config_value");
                        // Logic to read and print the a varable value
                        show_config_value(&key)
                            .await
                            .context("Failed to get variable value")?;
                    }
                    ConfigAction::List => {
                        info!("Config: LIST initiated");
                        trace!("Delegating to config::list::list_config");
                        // Logic to list all the variables
                        list_config().await?;
                    }
                    ConfigAction::Setup => {
                        info!("Config: SETUP initiated");
                        trace!("Delegating to config::setup::handle_setup");
                        handle_setup().await.context("Failed to run setup")?;
                    }
                }
            }
        }
        Ok(()) // Return success if no errors occurred
    }
}
