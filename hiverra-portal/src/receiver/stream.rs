use {
    crate::{
        history::ReceiveSummary, progress::ProgressManager, receiver::receive_item::receive_item,
    },
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
) -> (anyhow::Result<()>, ReceiveSummary) {
    let mut summary = ReceiveSummary {
        items: Vec::new(),
        total_bytes: 0,
    };
    // receive file or directories
    let reader: Box<dyn AsyncRead + Unpin + Send> = if compressed {
        debug!("Initializing Gzip decoder and Tar archive reader...");
        Box::new(GzipDecoder::new(BufReader::new(socket)))
    } else {
        debug!("Initializing Tar archive reader (no compression)...");
        Box::new(BufReader::new(socket))
    };
    let mut archive = Archive::new(reader);

    if let Err(err) = receive_item(&mut archive, target_dir, total_items, prog, &mut summary).await
    {
        return (Err(err), summary);
    }
    trace!("receive_item recursive loop completed.");

    debug!("Extraction complete. Recovering stream...");
    let _reader = archive.into_inner();
    trace!("Archive reader recovered.");

    (Ok(()), summary)
}
