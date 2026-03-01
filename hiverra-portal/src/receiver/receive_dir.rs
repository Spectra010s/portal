use {
    crate::metadata::DirectoryMetadata,
    anyhow::{Context, Result},
    std::path::PathBuf,
    tokio::{
        io::BufReader,
        net::TcpStream,
        fs::{create_dir_all, rename, remove_dir_all},
    },
    tokio_tar::Archive,
};

pub async fn receive_directory(socket: &mut TcpStream, target_dir: &PathBuf, meta: &DirectoryMetadata) -> Result<()> {

    //  Create a Temporary Path
    let temp_name = format!(".tmp_{}_portal", meta.dirname);
    let temp_path = target_dir.join(&temp_name);
    let final_path = target_dir.join(&meta.dirname);

    // Clean up any old failed attempts
    if temp_path.exists() {
        let _ = remove_dir_all(&temp_path).await;
    }
    create_dir_all(&temp_path).await?;

    // Unpack into Temp
    let reader = BufReader::new(socket);
    let mut archive = Archive::new(reader);
    archive.unpack(&temp_path).await.context("Failed to unpack TAR stream")?;

    //  Move to Final Destination
    rename(&temp_path, &final_path).await
        .context("Failed to move directory from temp to final destination")?;

    println!("Portal: Directory '{}' received successfully!", meta.dirname);
    Ok(())
}
