use {tokio::net::TcpStream,
anyhow::Result,
 std::path::PathBuf,
 crate::send_dir
 };

pub async fn sender(dir: PathBuf) -> Result<()> {
    // Connect to receiver
    let stream = TcpStream::connect("127.0.0.1:7878").await?;

    // Call send directory
    send_dir::send_directory(stream, &dir).await?;

    Ok(())
}