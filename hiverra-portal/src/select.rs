use {
    anyhow::{Context, Result},
    inquire::MultiSelect,
    std::path::PathBuf,
    tokio::fs::read_dir,
};

pub async fn select_files_to_send() -> Result<Option<Vec<PathBuf>>> {
    let mut entries = read_dir(".").await.context("Failed to read directory")?;
    let mut files = Vec::new();

    while let Ok(Some(entry)) = entries.next_entry().await {
        if let Ok(meta) = entry.metadata().await {
            if meta.is_file() {
                files.push(entry.path());
            }
        }
    }

    if files.is_empty() {
        println!("Portal: No files found in the current directory.");
        return Ok(None);
    }

    // Convert PathBufs to Strings for the UI display
    let options: Vec<String> = files
        .iter()
        .filter_map(|p| p.file_name())
        .map(|name| name.to_string_lossy().to_string())
        .collect();

    match MultiSelect::new(
        "Select files to send (Space to toggle, Enter to confirm):",
        options,
    )
    .prompt()
    {
        Ok(choices) => {
            if choices.is_empty() {
                return Ok(None);
            }
            // Convert the user's string choices back into PathBufs
            let selected_paths = choices.into_iter().map(PathBuf::from).collect();

            Ok(Some(selected_paths))
        }
        Err(_) => {
            println!("Portal: Selection cancelled.");
            Ok(None)
        }
    }
}
