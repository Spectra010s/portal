use {
std::path::PathBuf,
crate::metadata::{FileMetadata},
anyhow::{Context, Result},
tokio::{
        fs::File,
        io::{AsyncReadExt, AsyncWriteExt, BufReader},
        net::TcpStream
    },
    bincode::serialize,
};
    

    
pub async fn send_file(
stream: &mut TcpStream, path: &PathBuf, meta: &FileMetadata) -> Result<()> 

{

   let encoded_meta = serialize(&meta)?;
    stream.write_all(&(encoded_meta.len() as u32).to_be_bytes()).await?;
    stream.write_all(&encoded_meta).await?;
    

      let file_handle = File::open(path)
            .await
            .context("We found the file, but couldn't open it (it might be locked).")?;

        println!("Portal: Connection established to the file system.");

        let mut reader = BufReader::new(file_handle);
        println!("Portal: Buffer initialized and ready for streaming.");

        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = reader
                .read(&mut buffer)
                .await
                .context("Failed to read file")?;
            if bytes_read == 0 {
                break;
            }
            stream
                .write_all(&buffer[..bytes_read])
                .await
                .context("Failed to send file")?;
        }
    

    Ok(())
}