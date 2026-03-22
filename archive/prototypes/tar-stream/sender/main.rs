use  { std::path::PathBuf,
 inquire::Select,
 tokio::fs,
 anyhow::Result,
 };
mod sender;
mod send_dir;

#[tokio::main]
async fn main() -> Result<()> {

    let mut dirs = vec![];
    let mut entries = fs::read_dir(".").await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            dirs.push(path);
        }
    }

    if dirs.is_empty() {
        println!("No directories found in current folder.");
        return Ok(());
    }

    let options: Vec<String> = dirs.iter().filter_map(|p| p.file_name()).map(|name| name.to_string_lossy().to_string()).collect();
        
    
    let dir_selection = Select::new("Select a directory to send:", options)
        .prompt()?;
        
        
        let dir_select = PathBuf::from(dir_selection);
    // Call sender
    sender::sender(dir_select).await?;

    Ok(())
}