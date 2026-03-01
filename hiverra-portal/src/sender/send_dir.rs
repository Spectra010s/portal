use {
    crate::metadata::DirectoryMetadata,
    anyhow::Result,
    tokio::{
        fs::File,
        io::AsyncWriteExt,
        net::TcpStream,
    },
    async_compression::tokio::write::GzipEncoder,
    tokio_tar::{Builder, Header},
    std::path::PathBuf,
    bincode::serialize
};

pub async fn send_directory(
    stream: &mut TcpStream,
    base_path: &PathBuf,
    meta: &DirectoryMetadata,
) -> Result<()> {
    // Serialize and send directory metadata first
    let encoded_meta = serialize(meta)?;
    stream.write_all(&(encoded_meta.len() as u32).to_be_bytes()).await?;
    stream.write_all(&encoded_meta).await?;

    // Wrap TCP stream in gzip encoder
    let compressor = GzipEncoder::new(stream);
    let mut builder = Builder::new(compressor);

    // Append all files from metadata
    for file_meta in &meta.files {
        let local_path = base_path.join(&file_meta.filename);

        let mut file = File::open(&local_path).await?;
        let mut header = Header::new_gnu();
        header.set_path(&file_meta.filename)?;
        header.set_size(file_meta.file_size);
        header.set_mode(0o644);
        header.set_cksum();

        builder.append(&header, &mut file).await?;
    }

    // Finish tar and gzip
    builder.into_inner().await?.shutdown().await?;
    Ok(())
}