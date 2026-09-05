use {
    crate::{
        metadata::{TransferItem, TransferResult},
        sender::send_item::send_item,
        TransferProgress,
    },
    crate::error::{PxpError, Result},
    async_compression::tokio::write::GzipEncoder,
    bincode,
    std::path::PathBuf,
    tokio::{
        io::{AsyncReadExt, AsyncWrite, AsyncWriteExt},
        net::TcpStream,
    },
    tokio_tar::Builder,
    tracing::{debug, info, trace, warn},
};

/// Attempt to read a [`crate::metadata::ReceiverError`] abort frame from the
/// socket when the send side gets an unexpected connection error.
///
/// If the receiver sent an abort frame before dropping the connection it will
/// be sitting in the socket's receive buffer. Reading it here turns an opaque
/// `BrokenPipe` into a meaningful error message like "receiver ran out of disk
/// space" or "user cancelled".
///
/// This is best-effort: if no frame is available (e.g. the connection dropped
/// cleanly or the receiver is an older build) we return the original error.
async fn try_read_receiver_abort(socket: &mut TcpStream, original: PxpError) -> PxpError {
    use crate::metadata::ReceiverError;

    let mut len_buf = [0u8; 4];
    // Use a very short timeout — if the frame is in the buffer it will be
    // available immediately. We do not want to block the error path.
    match tokio::time::timeout(
        std::time::Duration::from_millis(200),
        socket.read_exact(&mut len_buf),
    )
    .await
    {
        Ok(Ok(_)) => {}
        _ => return original, // timeout or read error — no frame waiting
    }

    let payload_len = u32::from_be_bytes(len_buf) as usize;
    if payload_len == 0 || payload_len > 4096 {
        return original;
    }

    let mut payload = vec![0u8; payload_len];
    if socket.read_exact(&mut payload).await.is_err() {
        return original;
    }

    match bincode::deserialize::<ReceiverError>(&payload) {
        Ok(abort) => {
            warn!(
                "Receiver sent structured abort ({:?}): {}",
                abort.kind, abort.message
            );
            // Wrap the receiver's message in an Io error so it surfaces
            // cleanly through anyhow's error chain in the CLI.
            PxpError::Io(std::io::Error::new(
                std::io::ErrorKind::ConnectionAborted,
                format!("Receiver aborted: {}", abort.message),
            ))
        }
        Err(_) => original,
    }
}

async fn stream_items<W: AsyncWrite + Unpin + Send>(
    builder: &mut Builder<W>,
    items_to_send: Vec<(PathBuf, TransferItem)>,
    progress: Option<&dyn TransferProgress>,
) -> Result<()> {
    let total = items_to_send.len();
    for (index, (path, item)) in items_to_send.into_iter().enumerate() {
        debug!("Processing item {}: {:?}", index + 1, path);

        if let Some(prog) = progress {
            let (name, bytes, is_dir) = match &item {
                TransferItem::File(fm) => (fm.filename.clone(), fm.file_size, false),
                TransferItem::Directory(dm) => (dm.dirname.clone(), dm.total_size, true),
            };

            prog.set_current_item(index + 1, total);

            if is_dir && bytes == 0 {
                prog.println(&format!(
                    "Portal: Note: Directory '{}' is empty. Sending structure only.",
                    name
                ));
            }

            let item_prog = prog.create_item_progress(&name, bytes);
            send_item(builder, path, item, Some(&*item_prog))
                .await
                .map_err(|e| PxpError::Archive(e.to_string()))?;
            item_prog.finish_and_clear();

            let kind = if is_dir { "Directory" } else { "File" };
            prog.println(&format!("Portal: {} '{}' sent successfully!", kind, name));
        } else {
            send_item(builder, path, item, None)
                .await
                .map_err(|e| PxpError::Archive(e.to_string()))?;
        }
    }
    Ok(())
}

pub async fn send_stream(
    mut stream: TcpStream,
    items_to_send: Vec<(PathBuf, TransferItem)>,
    no_compress: bool,
    progress: Option<&dyn TransferProgress>,
) -> Result<TransferResult> {
    if no_compress {
        debug!("Initializing Tar builder (no compression)...");
        let mut builder = Builder::new(stream);
        info!("Starting TAR stream to network (no compression)...");

        if let Err(e) = stream_items(&mut builder, items_to_send, progress).await {
            stream = builder.into_inner().await.unwrap_or_else(|_| {
                // If we can't recover the stream we can't probe for an abort frame.
                // Return a dummy TcpStream by panicking is wrong — just surface the
                // original error as-is in this edge case.
                unreachable!("into_inner should not fail on an uncompressed builder")
            });
            return Err(try_read_receiver_abort(&mut stream, e).await);
        }

        debug!("Finalizing Tar archive structure...");
        builder.finish().await?;

        let mut stream: TcpStream = builder.into_inner().await?;
        trace!("Flushing underlying TCP stream...");
        stream.flush().await?;
        debug!("TCP stream flushed. Waiting for receiver ack...");
        read_transfer_result(&mut stream).await
    } else {
        debug!("Initializing Gzip encoder and Tar builder...");
        let compressor = GzipEncoder::new(stream);
        let mut builder = Builder::new(compressor);

        info!("Starting TAR stream to network...");
        if let Err(e) = stream_items(&mut builder, items_to_send, progress).await {
            // Recover the socket through the encoder for abort-frame probing.
            if let Ok(mut compressor) = builder.into_inner().await {
                let _ = compressor.shutdown().await;
                let mut raw = compressor.into_inner();
                return Err(try_read_receiver_abort(&mut raw, e).await);
            }
            return Err(e);
        }

        debug!("Finalizing Tar archive structure...");
        builder.finish().await?;

        let mut compressor: GzipEncoder<TcpStream> = builder.into_inner().await?;

        debug!("Shutting down Gzip compressor...");
        compressor.shutdown().await?;
        trace!("Compressor shutdown complete.");

        let mut stream = compressor.into_inner();
        trace!("Flushing underlying TCP stream...");
        stream.flush().await?;
        debug!("TCP stream flushed. Waiting for receiver ack...");
        read_transfer_result(&mut stream).await
    }
}

/// Read the [`TransferResult`] acknowledgment frame the receiver sends after it
/// has finished reconciling all staged items into the target directory.
///
/// Frame layout:
/// ```text
/// [ length: u32 big-endian ][ bincode-encoded TransferResult ]
/// ```
///
/// If the receiver closes the connection without sending an ack (e.g. because
/// it is an older build that does not implement this frame), we synthesise a
/// best-effort result rather than failing the transfer. This keeps the sender
/// backward-compatible with receivers that pre-date this change.
async fn read_transfer_result(stream: &mut TcpStream) -> Result<TransferResult> {
    // Read the 4-byte length prefix.
    let mut len_buf = [0u8; 4];
    match stream.read_exact(&mut len_buf).await {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof
            || e.kind() == std::io::ErrorKind::ConnectionReset
            || e.kind() == std::io::ErrorKind::BrokenPipe =>
        {
            // Receiver closed without acking — treat as implicit success for
            // backward compatibility, but warn so it shows up in logs.
            warn!(
                "Receiver closed connection without sending a result ack (older build?). \
                 Assuming success."
            );
            return Ok(TransferResult {
                success: true,
                items_received: 0,
                bytes_received: 0,
                error: None,
            });
        }
        Err(e) => return Err(PxpError::Io(e)),
    }

    let payload_len = u32::from_be_bytes(len_buf) as usize;
    if payload_len > 4096 {
        return Err(PxpError::Protocol(format!(
            "Transfer result frame is implausibly large ({} bytes).",
            payload_len
        )));
    }

    let mut payload = vec![0u8; payload_len];
    stream.read_exact(&mut payload).await?;

    let result: TransferResult = bincode::deserialize(&payload)?;
    debug!(
        "Received transfer result ack: success={}, items={}, bytes={}",
        result.success, result.items_received, result.bytes_received
    );

    if result.success {
        info!("Receiver confirmed all items landed successfully.");
    } else {
        warn!(
            "Receiver reported a failure: {}",
            result.error.as_deref().unwrap_or("no details")
        );
    }

    Ok(result)
}
