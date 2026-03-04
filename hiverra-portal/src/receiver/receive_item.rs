use {
    anyhow::{Context, Result},
    async_compression::tokio::bufread::GzipDecoder,
    std::path::PathBuf,
    tokio::{
        fs::{create_dir_all, remove_dir_all, rename},
        io::BufReader,
        net::TcpStream,
    },
    tokio_tar::Archive,
};
pub async fn receive_item(
    socket: &mut TcpStream,
    target_dir: &PathBuf,
    item_name: &str,
    is_dir: bool,
) -> Result<()> {
    // Setup Atomic Paths
    let temp_name = format!(".tmp_{}_portal", item_name);
    let temp_path = target_dir.join(&temp_name);
    let final_path = target_dir.join(item_name);

    // Clean up old failed attempts
    if temp_path.exists() {
        let _ = remove_dir_all(&temp_path).await;
    }
    create_dir_all(&temp_path).await?;

    //  Unpack the compressed TAR stream into Temp
    // We wrap the socket in Gzip, then Tar
    let reader = BufReader::new(socket);
    let decoder = GzipDecoder::new(reader);
    let mut archive = Archive::new(decoder);

    archive
        .unpack(&temp_path)
        .await
        .context(format!("Failed to unpack {} to temp", item_name))?;

    //  Move to Final Destination
    // If it's a file, we need to point to the file inside the temp folder
    if !is_dir {
        let actual_file_in_temp = temp_path.join(item_name);
        rename(&actual_file_in_temp, &final_path).await?;
        let _ = remove_dir_all(&temp_path).await; // Cleanup the empty temp dir
    } else {
        rename(&temp_path, &final_path)
            .await
            .context("Failed to move directory to final destination")?;
    }

    let kind_label = if is_dir { "Directory" } else { "File" };
    println!(
        "Portal: {} '{}' received successfully!",
        kind_label, item_name
    );

    Ok(())
}
