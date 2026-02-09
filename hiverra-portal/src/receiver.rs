use std::{
    fs::File,
    io::{Read, Write},
    net::TcpListener,
};
// Import anyhow for add descriptive error handling
use crate::metadata::FileMetadata;
use anyhow::{Context, Result};
use bincode::deserialize;
use network_interface::{NetworkInterface, NetworkInterfaceConfig};

fn get_local_ip() -> Option<String> {
    NetworkInterface::show()
        .ok()?
        .into_iter()
        .find(|itf| {
            let name = itf.name.to_lowercase();
            name.contains("wlan") || name.contains("eth") || name.contains("en")
        })
        .and_then(|itf| {
            itf.addr
                .into_iter()
                .find(|addr| addr.ip().is_ipv4() && !addr.ip().is_loopback())
                .map(|addr| addr.ip().to_string())
        })
}

pub fn receive_file() -> Result<()> {
    println!("Portal: Initializing  systems...");
    println!("Portal: Getting IP address");

    let my_ip = get_local_ip().context("Failed to get IP address, pls try again")?;

    let listener = TcpListener::bind("0.0.0.0:7878").context("Failed to bind to port 7878")?;

    println!("Portal: crearing wormhole at {:?}", my_ip);
    println!(
        "Portal: on the sender machine, run: portal send <file> -a {}",
        my_ip
    );

    println!("Receiver: Portal open. Waiting for a connection on port 7878...");

    let (mut socket, addr) = listener.accept().context("Failed to accept connection")?;
    println!("Receiver: Connection established with {}!", addr);
    println!("Portal: Connected to sender");
    println!("Portal: Waiting for incoming files...");

    // 1. Read the metadata length
    let mut metadata_len_buf = [0u8; 4];
    socket
        .read_exact(&mut metadata_len_buf)
        .context("Failed to read metadata length")?; // Read exactly 4 bytes

    let metadata_len = u32::from_be_bytes(metadata_len_buf) as usize;

    // 2. Read the Metadata Blob

    let mut metadata_buf = vec![0u8; metadata_len];
    socket
        .read_exact(&mut metadata_buf)
        .context("Failed to read metadata blob")?;

    // 3 Turn those bytes into our Struct
    let file_info: FileMetadata =
        deserialize(&metadata_buf).context("Failed to deserialize metadata")?;

    // Read the name and size and ?description
    let filename = &file_info.filename;
    let file_size = file_info.file_size;
    let description = &file_info.description;

    println!("Receiving file: {} ({} bytes)", filename, file_size);

    if let Some(desc) = description {
        println!("Portal: Sender left a note: \"{}\"", desc);
    } else {
        println!("Portal: No description provided for this transfer.");
    }

    // 4. Create the file on disk
    // filename is now a &String, we dont need the &* anymore.
    let mut out_file = File::create(filename).context("Failed to create file on disk")?;

    // 5. The loop to actually save the bytes
    let mut buffer = [0u8; 8192];
    let mut received_so_far = 0;

    while received_so_far < file_size {
        let bytes_read = socket
            .read(&mut buffer)
            .context("Network read error during file transfer")?;
        if bytes_read == 0 {
            break;
        } // Sender hung up

        out_file
            .write_all(&buffer[..bytes_read])
            .context("Disk write error")?;
        received_so_far += bytes_read as u64;
    }

    println!("Portal: Transfer complete! Saved as {}", filename);

    Ok(())
}
