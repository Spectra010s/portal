use  {
tokio::fs::{create_dir_all, rename, remove_dir_all},
 tokio::io::BufReader,
 tokio::net::TcpStream,
 tokio_tar::Archive,
 std::path::PathBuf,
 anyhow::Result
 };

pub async fn receive_directory(socket: TcpStream, target_dir: &PathBuf) -> Result<()> {
    // Temporary folder creation 
    let temp_path = target_dir.join(".tmp_portal");
    if temp_path.exists() {
        let _ = remove_dir_all(&temp_path).await;
    }
    create_dir_all(&temp_path).await?;

    // Wrap the TCP stream in a BufReader and extract tar archive
    let reader = BufReader::new(socket);
    let mut archive = Archive::new(reader);
    archive.unpack(&temp_path).await?;

    // Move to final destination
    let final_path = target_dir.join("received_dir");
    rename(&temp_path, &final_path).await?;

    println!("Receiver: Directory received successfully!");
    Ok(())
}