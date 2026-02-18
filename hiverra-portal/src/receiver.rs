use {
    crate::discovery::beacon::start_beacon,
    crate::{config::models::PortalConfig, metadata::TransferManifest},
    anyhow::{Context, Result, anyhow},
    bincode::deserialize,
    home::home_dir,
    inquire::Text,
    network_interface::{NetworkInterface, NetworkInterfaceConfig},
    std::{net::IpAddr, path::PathBuf},
    tokio::{
        fs::{File, create_dir_all},
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    },
};

/// Searches for the first available IPv4 address on common Wi-Fi interface names
async fn get_local_ip() -> Option<String> {
    // Retrieve all network interfaces
    let interfaces = NetworkInterface::show().ok()?;

    for interface in interfaces {
        let name = interface.name.to_lowercase();

        // Comprehensive check
        if name.contains("wlan")
            || name.contains("wlp")
            || name.contains("wi-fi")
            || name.starts_with("en")
        {
            for addr in interface.addr {
                // Filter for IPv4 and ignore loopback with check
                if let IpAddr::V4(ipv4) = addr.ip() {
                    if !ipv4.is_loopback() {
                        return Some(ipv4.to_string());
                    }
                }
            }
        }
    }
    None
}

pub async fn receive_file(port: Option<u16>, dir: &Option<PathBuf>) -> Result<()> {
    println!("Portal: Initializing  systems...");
    println!("Portal: Getting IP address");

    let my_ip = get_local_ip()
        .await
        .context("Failed to get IP address, pls try again")?;

    // Use the CLI flag directly
    let n_port = if let Some(port) = port {
        println!("Portal: Overriding config port with CLI port...");
        port
    } else if let Some(cfg) = PortalConfig::load_or_return().await? {
        //  Use config if it exists and has a value
        if let Some(p) = cfg.network.default_port {
            println!("Portal: Port not given, using config port...");
            p
        } else {
            return Err(anyhow!("No port provided and config has no port set"));
        }
    } else {
        //  Neither CLI nor config
        return Err(anyhow!("No port provided and no config found"));
    };

    // Fetching username with load_all
    let full_cfg = PortalConfig::load_all()
        .await
        .context("Failed to load user config")?;

    let username = full_cfg.user.username.ok_or_else(|| {
        anyhow!("No username found. Please run 'portal config set user.username <name>' first.")
    })?;

    // Unique session ID for this transfer
    let node_id = uuid::Uuid::new_v4().to_string();

    let new_addr = format!("0.0.0.0:{}", n_port);

    let listener = TcpListener::bind(&new_addr)
        .await
        .context("Failed to bind to port")?;

    println!("Portal: Creating wormhole at {:?}", my_ip);
    println!("Portal: Wormhole open for {:?}", username);

    // The Tokio Select logic to run Beacon and Listener together
    let (mut socket, addr) = tokio::select! {
        // start the beacon
        _ = start_beacon(username, node_id.clone(), n_port) => {
          println!("Portal: Beacon active. Waiting for sender...");
            return Err(anyhow!("Portal: Discovery beacon stopped unexpectedly"));
        }
        // wait for the actual TCP connection
        result = listener.accept() => {
            result.context("Failed to accept connection")?
        }
    };

    println!("Portal: Connection established with {}!", addr);
    println!("Portal: Connected to sender");
    println!("Portal: Waiting for incoming files...");

    // Send ID to Sender so they can verify who we are
    let id_bytes = node_id.as_bytes();
    let id_len = id_bytes.len() as u32;

    socket
        .write_all(&id_len.to_be_bytes())
        .await
        .context("Failed to send verification length")?;
    socket
        .write_all(id_bytes)
        .await
        .context("Failed to send verification ID")?;

    //  Read the metadata length
    let mut manifest_len_buf = [0u8; 4];
    socket
        .read_exact(&mut manifest_len_buf)
        .await
        .context("Failed to read manifest length")?;

    let manifest_len = u32::from_be_bytes(manifest_len_buf) as usize;

    //  Read the Metadata Blob

    let mut manifest_buf = vec![0u8; manifest_len];
    socket
        .read_exact(&mut manifest_buf)
        .await
        .context("Failed to read manifest blob")?;

    // Deserialize the manifest
    let file_manifest: TransferManifest =
        deserialize(&manifest_buf).context("Failed to deserialize manifest")?;
    println!("Portal: Manifest received.");
    let files_infos = &file_manifest.files;
    let total_files = file_manifest.total_files;
    let description = &file_manifest.description;

    // Print basic info for the user
    println!("Portal: Incoming transfer - {} file(s)", total_files);

    if let Some(desc) = description {
        println!("Portal: Sender left a note: \"{}\"", desc);
    } else {
        println!("Portal: No description provided for this transfer.");
    }

    // Determine the directory to save files
    // Use CLI-provided path, or config, or prompt user if neither exists
    let target_dir: PathBuf = if let Some(dir) = dir {
        dir.clone()
    } else if let Some(cfg) = PortalConfig::load_or_return().await? {
        if let Some(d) = &cfg.storage.download_dir {
            println!("Portal: Using directory from config: {}", d.display());
            d.clone()
        } else {
            println!("Portal: Config exists but download directory not set.");
            let default_path = home_dir()
                .ok_or_else(|| anyhow!("Could not find home directory"))?
                .join("Downloads")
                .display()
                .to_string();

            let dir_string = Text::new("Portal: Where should Portal save this file?")
                .with_default(&default_path)
                .with_help_message("Enter a valid folder path.")
                .prompt()
                .context("No directory provided")?;

            PathBuf::from(dir_string)
        }
    } else {
        let default_path = home_dir()
            .ok_or_else(|| anyhow!("Could not find home directory"))?
            .join("Downloads")
            .display()
            .to_string();

        let dir_string = Text::new("Portal: Where should Portal save this file?")
            .with_default(&default_path)
            .with_help_message("Enter a valid folder path.")
            .prompt()
            .context("No directory provided")?;

        PathBuf::from(dir_string)
    };
    // Ensure the directory exists
    create_dir_all(&target_dir)
        .await
        .context("Failed to create target directory")?;

    //  Loop through each file in the manifest
    for (index, file_info) in files_infos.iter().enumerate() {
        let filename = &file_info.filename;
        let file_size = file_info.file_size;

        println!(
            "Portal: Receiving file {} of {}: '{}' ({} bytes)",
            index + 1,
            total_files,
            filename,
            file_size
        );

        let file_path = target_dir.join(filename);
        let mut out_file = File::create(&file_path)
            .await
            .context("Failed to create file on disk")?;

        let mut buffer = [0u8; 8192];
        let mut received_so_far = 0;

        while received_so_far < file_size {
            let bytes_read = socket
                .read(&mut buffer)
                .await
                .context("Network read error during file transfer")?;
            if bytes_read == 0 {
                break; // Sender hung up
            }

            out_file
                .write_all(&buffer[..bytes_read])
                .await
                .context("Disk write error")?;

            received_so_far += bytes_read as u64;
        }

        println!("Portal: File '{}' received successfully!", filename);
    }

    println!(
        "Portal: All file(s) have been received successfully! and saved to '{}'",
        target_dir.display()
    );

    Ok(())
}
