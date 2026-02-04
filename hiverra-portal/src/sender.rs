use std::{
    fs::{File, metadata},
    io::{BufReader, Read, Write},
    net::TcpStream,
    path::PathBuf,
};

pub fn send_file(file: &PathBuf) {
    // Check 1. check if the path exists before attempting to send
    if !file.exists() {
        println!("Error: The file '{}' does not exist.", file.display());
    } else if file.is_dir() {
        // Check 2: Is it a folder
        // We don't want to "send" a folder yet because folders
        // need to be zipped or recursed
        println!(
            "Error: '{}' is a directory. Portal only supports single files right now.",
            file.display()
        );
    } else {
        // If it exists AND it's not a directory, it's a file!
        // Get the metadata (size, permissions, etc.)
        let file_info = metadata(file).expect("Failed to read metadata");
        // chaange to file_size for clarity
        let file_size = file_info.len();
        // Size in bytes
        // Open the file for reading
        let Ok(file_handle) = File::open(file) else {
            println!("Error: We found the file, but couldn't open it (it might be locked).");
            return;
        };

        println!("Portal: File found!");

        println!("Portal: Connection established to the file system.");
        println!(
            "Portal: Preparing to send '{}' ({} bytes)...",
            file.display(),
            file_size
        );
        let mut reader = BufReader::new(file_handle);

        println!("Portal: Buffer initialized and ready for streaming.");
        // Prepare the Metadata Header
        // Get the filename and size
        let filename = file.file_name().expect("Error reading name").to_str();

        // We'll send: [Name Length (4 bytes)] + [The Name] + [The File Size (8 bytes)]
        let name_bytes = filename.expect("Error occured").as_bytes();
        let name_len: u32 = name_bytes.len().try_into().expect("Filename too long");

        let mut stream =
            TcpStream::connect("127.0.0.1:7878").expect("Could not connect to Reciever!");
        println!("Sender: Connected to receiver!");

        // 3. Stream the Metadata to the Pipe
        stream.write_all(&name_len.to_be_bytes()); // Send name length first
        stream.write_all(name_bytes); // Send the actual name second
        stream.write_all(&file_size.to_be_bytes()); // Send the total file size
        let mut buffer = [0u8; 8192];
        // 4. NOW start the File Loop we discussed
        println!("Portal: Sending {}...", file.display());
        loop {
            let bytes_read = reader.read(&mut buffer).expect("Failed to read file");
            if bytes_read == 0 {
                break;
            }
            stream
                .write_all(&buffer[..bytes_read])
                .expect("Failed to send file");
        }

        println!("Portal: {} sent successfuly!", file.display());
    }
}
