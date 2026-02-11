use anyhow::{Context, Result};
use inquire::Select;
use tokio::fs::read_dir;

pub async fn select_file_to_send() -> Result<Option<String>> {
    // 1. Open the current directory
    let mut entries = read_dir(".").await.context("Failed to read directory")?;
    let mut files = Vec::new();

    // 2. Collect file names into the vector
    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();

        if let Ok(file_metadata) = entry.metadata().await {
            if file_metadata.is_file() {
                // Safely get the file name
                if let Some(name) = path.file_name() {
                    files.push(name.to_string_lossy().into_owned());
                }
            }
        }
    }

    // 3. Handle empty directory
    if files.is_empty() {
        println!("Portal: No files found in the current directory.");
        return Ok(None);
    }

    // 4. Show the interactive picker
    // Inquire handles the terminal UI logic here
    match Select::new("Select a file to send:", files).prompt() {
        Ok(choice) => Ok(Some(choice)),
        Err(_) => {
            println!("Portal: Selection cancelled.");
            Ok(None)
        }
    }
}
