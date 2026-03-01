use {
    crate::metadata::FileMetadata,
    anyhow::{Context, Result},
    std::path::PathBuf,
    tokio::{
        fs::File,
        io::{AsyncWriteExt,AsyncReadExt},
        net::TcpStream,
    },
};

pub async fn receive_file(socket: &mut TcpStream, target_dir: &PathBuf, meta: &FileMetadata) -> Result<()> {
  
    
    // Create file and stream bytes
    let file_path = target_dir.join(&meta.filename);
    let mut out_file = File::create(&file_path).await.context("Failed to create file on disk")?;
    
    let mut received = 0;
    let mut buffer = [0u8; 8192];

    while received < meta.file_size {
        let bytes_read = socket.read(&mut buffer).await.context("Network read error during file transfer")?;
        if bytes_read == 0 { break; }
        
        out_file.write_all(&buffer[..bytes_read]).await                .context("Disk write error")?;
        received += bytes_read as u64;
    }
    
    println!("Portal: File '{}' received successfully!", meta.filename);

    Ok(())
}
