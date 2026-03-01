use {
tokio::net::TcpStream,
 tokio_tar::Builder,
 std::path::PathBuf,
 anyhow::Result
 };

pub async fn send_directory( stream: TcpStream, dir_path: &PathBuf) -> Result<()> {
    // Wrap the TCP stream in Builder
    let mut builder = Builder::new(stream);

    // Append the directory recursively
    // found this new one append dir all
    builder.append_dir_all(".", dir_path).await?;

    // Finish the tar archive
    builder.into_inner().await?;

    println!("Sender: Directory sent successfully!");
    Ok(())
}