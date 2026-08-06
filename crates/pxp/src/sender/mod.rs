mod handshake;
pub mod manifest;
pub(crate) mod send_item;
mod stream;

pub use handshake::{connect_to_receiver, discover_receiver};
pub use manifest::{create_directory_metadata, create_file_metadata, create_global_transfer_manifest};
pub use stream::send_stream;

use {
    crate::metadata::GlobalTransferManifest,
    crate::error::Result,
    tokio::{io::AsyncWriteExt, net::TcpStream},
    tracing::debug,
};

/// Serialize and send the global transfer manifest over the TCP stream.
pub async fn send_manifest(
    stream: &mut TcpStream,
    manifest: &GlobalTransferManifest,
) -> Result<()> {
    let encoded = bincode::serialize(manifest)?;
    let manifest_len = encoded.len() as u32;
    debug!("Sending serialized global manifest ({} bytes)...", manifest_len);
    stream.write_all(&manifest_len.to_be_bytes()).await?;
    stream.write_all(&encoded).await?;
    Ok(())
}
