use {
    crate::metadata::TransferItem,
    anyhow::Result,
    async_compression::tokio::write::GzipEncoder,
    bincode::serialize,
    std::path::PathBuf,
    tokio::{fs::File, io::AsyncWriteExt, net::TcpStream},
    tokio_tar::{Builder, Header},
};

pub async fn send_item(
    mut stream: TcpStream,
    path: PathBuf,
    item: TransferItem,
) -> Result<TcpStream> {
    //  Send Metadata for that item
    let encoded_meta = serialize(&item)?;
    stream
        .write_all(&(encoded_meta.len() as u32).to_be_bytes())
        .await?;
    stream.write_all(&encoded_meta).await?;

    //  Wrap in Gzip Compression
    let compressor = GzipEncoder::new(stream);

    //  Tar builder with Gzip
    let mut builder = Builder::new(compressor);

    match &item {
        TransferItem::File(file_meta) => {
            println!(
                "Portal: Preparing to send '{}' ({} bytes)...",
                file_meta.filename, file_meta.file_size
            );

            let mut file = File::open(&path).await?;
            let mut header = Header::new_gnu();
            header.set_path(&file_meta.filename)?;
            header.set_size(file_meta.file_size);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append(&header, &mut file).await?;
        }
        TransferItem::Directory(dir_meta) => {
            println!(
                "Portal: Preparing to send directory '{}' ({} bytes)...",
                dir_meta.dirname, dir_meta.total_size
            );

            for file_meta in &dir_meta.files {
                let local_path = path.join(&file_meta.filename);
                let mut file = File::open(&local_path).await?;

                let mut header = Header::new_gnu();
                header.set_path(&file_meta.filename)?;
                header.set_size(file_meta.file_size);
                header.set_mode(0o644);
                header.set_cksum();
                builder.append(&header, &mut file).await?;
            }
        }
    }

    // call finish tar
    builder.finish().await?;

    // get the gzip after finish Tar
    let mut compressor = builder.into_inner().await?;

    // Finish Gzip
    compressor.shutdown().await?;

    // Recover original TcpStream
    let stream = compressor.into_inner();

    // Final Success Prints
    match &item {
        TransferItem::File(file_meta) => {
            println!("Portal: File '{}' sent successfully!", file_meta.filename);
        }
        TransferItem::Directory(dir_meta) => {
            println!(
                "Portal: Directory '{}' sent successfully!",
                dir_meta.dirname
            );
        }
    }

    Ok(stream)
}
