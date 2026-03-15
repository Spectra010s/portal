use {
    anyhow::{Context, Result},
    inquire::MultiSelect,
    std::path::PathBuf,
    tokio::fs::read_dir,
    tracing::{debug, info, trace, warn},
};

pub async fn select_files_to_send() -> Result<Option<Vec<PathBuf>>> {
    debug!("Scanning current directory for files...");
    let mut entries = read_dir(".").await.context("Failed to read directory")?;
    let mut files = Vec::new();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        trace!("Discovered potential file path: {:?}", path);
        files.push(path);
    }

    if files.is_empty() {
        warn!("User attempted to send files, but directory is empty.");
        println!("Portal: No files found in the current directory.");
        return Ok(None);
    }

    // Convert PathBufs to Strings for the UI display
    let options: Vec<String> = files
        .iter()
        .filter_map(|p| p.file_name())
        .map(|name| name.to_string_lossy().to_string())
        .collect();

    debug!("Found {} potential files to send", options.len());

    match MultiSelect::new(
        "Select files to send (Space to toggle, Enter to confirm):",
        options,
    )
    .prompt()
    {
        Ok(choices) => {
            if choices.is_empty() {
                info!("User completed selection but chose 0 files.");
                return Ok(None);
            }
            // Convert the user's string choices back into PathBufs
            trace!(
                "Converting {} selected strings back to PathBufs",
                choices.len()
            );
            let selected_paths: Vec<PathBuf> = choices.into_iter().map(PathBuf::from).collect();

            info!("User selected {} files for sending", selected_paths.len());
            debug!("Selected paths: {:?}", selected_paths);

            Ok(Some(selected_paths))
        }
        Err(e) => {
            info!("File selection UI cancelled by user or encountered error.");
            trace!("inquire::MultiSelect prompt error: {}", e);
            println!("Portal: Selection cancelled.");
            Ok(None)
        }
    }
}
