mod send_dir;
mod send_file;

use {
    crate::{
        discovery::listener::find_receiver,
        metadata::{
            DirectoryMetadata, FileMetadata, GlobalTransferManifest, ItemKind, TransferHeader,
            TransferItem,
        },
        select::select_files_to_send,
    },
    anyhow::{Context, Result, anyhow, Error},
    bincode::serialize,
    inquire::{Confirm, Text},
    std::path::PathBuf,
    std::time::Duration,
    tokio::{
        fs::metadata,
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpStream,
        task,
        time::timeout,
    },
    send_dir::send_directory,
    send_file::send_file,
    walkdir::WalkDir,
};


async fn create_file_metadata(path: &PathBuf) -> Result<FileMetadata> {
    let attr = metadata(path).await?;

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown_file")
        .to_string();

    Ok(FileMetadata {
        filename,
        file_size: attr.len(),
    })
}

async fn create_directory_metadata(dir: &PathBuf) -> Result<DirectoryMetadata> {
    let dir_clone = dir.clone();

    // using blocking thread bcus of walkdir
    let result = task::spawn_blocking(move || {
        let mut files_meta = Vec::new();
        let mut total_size = 0u64;

        for entry in WalkDir::new(&dir_clone)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path().to_path_buf();

                let attr = std::fs::metadata(&path)?;
                let size = attr.len();
                total_size += size;

                // Keep the folder structure relative to the parent
                let rel_path = path.strip_prefix(&dir_clone)?.to_string_lossy().to_string();

                files_meta.push(FileMetadata {
                    filename: rel_path,
                    file_size: size,
                });
            }
        }

        Ok::<(Vec<FileMetadata>, u64), Error>((files_meta, total_size))
    })
    .await??;

    let (files_meta, total_size) = result;

    Ok(DirectoryMetadata {
        dirname: dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown_dir")
            .to_string(),
        total_size,
        files: files_meta,
    })
}

pub async fn create_global_transfer_manifest(
    files: u32,
    dirs: u32,
    desc: Option<String>,
) -> Result<GlobalTransferManifest> {
    Ok(GlobalTransferManifest {
        total_files: files,
        total_directories: dirs,
        description: desc,
    })
}

pub async fn start_send(
    file: &Option<Vec<PathBuf>>,
    addr: &Option<String>,
    port: &u16,
    to: &Option<String>,
    recursive: &bool,
) -> Result<()> {
    let files = match file {
        Some(path) => path.clone(),
        None => {
            if let Ok(Some(selected)) = select_files_to_send().await {
                selected.clone()
            } else {
                return Ok(());
            }
        }
    };

    for file in &files {
        if !file.exists() {
            return Err(anyhow!(
                "File or directory '{}' does not exist",
                file.display()
            ));
        }
        if file.is_dir() {
            if !recursive {
                return Err(anyhow!(
                    "-r not specified; omitting directory '{}'",
                    file.display(),
                ));
            }
        }
    }

    //  New username discovery connection Logic
    let (target_ip, target_node_id, target_port) = if let Some(direct_addr) = addr {
        // Manual override
        (direct_addr.clone(), None, *port)
    } else {
        // Discovery Mode
        let target_username = match to {
            Some(username) => username.clone(),
            None => Text::new("Portal: Enter Receiver's username:")
                .prompt()
                .context("Failed to get username")?,
        };

        println!("Portal: Searching for receiver...: {}", target_username);

        let discovery_result = timeout(
        Duration::from_secs(30),
        find_receiver(&target_username)
    ).await.context("Portal: Search timed out. Make sure the receiver is active and on the same network.")??;

        let (ip, id, p) = discovery_result;
        (ip, Some(id), p)
    };

    let r_addr = format!("{}:{}", target_ip, target_port);
    println!("Portal: Connecting to {}...", r_addr);

    let mut stream = TcpStream::connect(&r_addr)
        .await
        .context("Could not connect to Receiver!")?;
    println!("Portal: Connection established!");

    // Read the ID the receiver is claiming
    let mut id_len_buf = [0u8; 4];
    stream.read_exact(&mut id_len_buf).await?;
    let id_len = u32::from_be_bytes(id_len_buf) as usize;

    let mut id_buf = vec![0u8; id_len];
    stream.read_exact(&mut id_buf).await?;
    let claimed_id = String::from_utf8(id_buf)?;

    // Verify it matches what we heard in the beacon
    if let Some(expected_id) = target_node_id {
        println!("Portal: Verifying identity...");
        if claimed_id != expected_id {
            return Err(anyhow!("Portal Security: ID mismatch! Connection aborted."));
        }
        println!("Portal: Identity verified. Starting transfer...");
    } else {
        println!(
            "Portal: Connected to {} (Manual mode: Identity check skipped).",
            target_ip
        );
    }

    //  Ask  user if to add a description
    let user_desc = if Confirm::new("Portal: Add description for this transfer?")
        .with_default(false)
        .prompt()?
    {
        Some(Text::new("Portal: Enter transfer description:").prompt()?)
    } else {
        None
    };

    //  Collect all files and directories
    let mut items_to_send: Vec<(PathBuf, TransferItem)> = Vec::new();

    for path in &files {
        if path.is_dir() {
            let dir_meta = create_directory_metadata(path).await?;
            items_to_send.push((path.clone(), TransferItem::Directory(dir_meta)));
        } else {
            let file_meta = create_file_metadata(path).await?;
            items_to_send.push((path.clone(), TransferItem::File(file_meta)));
        }
    }

    let (file_items, dir_items) = items_to_send
        .iter()
        .fold((0u32, 0u32), |(f, d), (_, item)| match item {
            TransferItem::File(_) => (f + 1, d),
            TransferItem::Directory(_) => (f, d + 1),
        });

    //  Create global manifest
    let global_manifest = create_global_transfer_manifest(file_items, dir_items, user_desc).await?;

    // Serialize and send global manifest
    let encoded_global = serialize(&global_manifest)?;
    stream
        .write_all(&(encoded_global.len() as u32).to_be_bytes())
        .await?;
    stream.write_all(&encoded_global).await?;

    println!("Portal: Global Manifest sent.");

    if let Some(d) = &global_manifest.description {
        println!("Portal: Note: {}", d);
    }
    // Send files sequentially
    let total_items = items_to_send.len();

    println!("Portal: Preparing to send {} items(s)...", total_items);

    // Send files snd directories
    for (index, (path, item)) in items_to_send.iter().enumerate() {
        let kind = match item {
            TransferItem::File(_) => ItemKind::File,
            TransferItem::Directory(_) => ItemKind::Directory,
        };

    

        let header = TransferHeader { kind };
        let encoded_header = serialize(&header)?;
        stream
            .write_all(&(encoded_header.len() as u32).to_be_bytes())
            .await?;
        stream.write_all(&encoded_header).await?;
        println!("Portal: Sending header: {:?}", header.kind);

        match item {
            TransferItem::File(file_meta) => {
                println!(
                    "Portal: Preparing to send '{}' ({} bytes)...",
                    file_meta.filename, file_meta.file_size
                );
                println!("Portal: Sending item {} of {}", index + 1, total_items);
                send_file(&mut stream, path, file_meta).await?;
                println!("Portal: File '{}' sent successfully!", file_meta.filename);
            }
            TransferItem::Directory(dir_meta) => {
                println!(
                    "Portal: Preparing to send directory '{}' ({} bytes)...",
                    dir_meta.dirname, dir_meta.total_size
                );
                println!("Portal: Sending item {} of {}", index + 1, total_items);
                send_directory(&mut stream, path, dir_meta).await?;
                println!(
                    "Portal: Directory '{}' sent successfully!",
                    dir_meta.dirname
                );
            }
        }
    }

    println!("Portal: All file(s) have been sent successfully!");
    Ok(())
}
