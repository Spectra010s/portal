use {
    crate::{
        history::ReceiveSummary,
        progress::ProgressManager,
        receiver::receive_item::receive_item,
    },
    anyhow::Result,
    async_compression::tokio::bufread::GzipDecoder,
    std::path::PathBuf,
    tokio::{
        io::{AsyncRead, BufReader},
        net::TcpStream,
    },
    tokio_tar::Archive,
    tracing::{debug, trace},
};

pub async fn receive_stream(
    socket: TcpStream,
    compressed: bool,
    target_dir: &PathBuf,
    total_items: u32,
    prog: Option<ProgressManager>,
) -> Result<ReceiveSummary> {
    // receive file or directories
    let reader: Box<dyn AsyncRead + Unpin + Send> = if compressed {
        debug!("Initializing Gzip decoder and Tar archive reader...");
        Box::new(GzipDecoder::new(BufReader::new(socket)))
    } else {
        debug!("Initializing Tar archive reader (no compression)...");
        Box::new(BufReader::new(socket))
    };
    let mut archive = Archive::new(reader);

    let summary = receive_item(&mut archive, target_dir, total_items, prog).await?;
    trace!("receive_item recursive loop completed.");

    debug!("Extraction complete. Recovering stream...");
    let _reader = archive.into_inner();
    trace!("Archive reader recovered.");

    Ok(summary)
}
