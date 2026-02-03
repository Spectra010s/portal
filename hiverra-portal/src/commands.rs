use clap::Subcommand;
use std::{
    fs::{File, metadata},
    io::{BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
};

// 2. Defining the Choices (The Enum)
// 'pub' makes this visible to main.rs
#[derive(Subcommand)]
pub enum Commands {
    /// Send a file
    Send {
        file: PathBuf, // changed it to PathBuf, so as to hold "File System Object".
    },
    /// Receive a file
    Receive,
}

impl Commands {
    // This is the method attached to the Enum
    // pub is needed here also to be able to call the function
    pub fn execute(&self) {
        match self {
            Commands::Send { file } => {
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
                        println!(
                            "Error: We found the file, but couldn't open it (it might be locked)."
                        );
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

                    let mut stream = TcpStream::connect("127.0.0.1:7878")
                        .expect("Could not connect to Reciever!");
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
            Commands::Receive => {
                println!("Portal: Initializing  systems...");
                let listener =
                    TcpListener::bind("127.0.0.1:7878").expect("Failed to bind to port 7878");

                println!("Receiver: Portal open. Waiting for a connection on port 7878...");
                let (mut socket, addr) = listener.accept().expect("Failed to accept connection");
                println!("Receiver: Connection established with {}!", addr);
                println!("Portal: Connected to sender");
                println!("Portal: Waiting for incoming files...");

                // 1. Read the name length (We sent a u32, which is 4 bytes)
                let mut name_len_buf = [0u8; 4];
                socket
                    .read_exact(&mut name_len_buf)
                    .expect("Failed to read name length"); // Read exactly 4 bytes
                let name_len = u32::from_be_bytes(name_len_buf); // Turn bytes back into a number

                // 2. Read the actual name
                let mut name_buf = vec![0u8; name_len as usize];
                socket.read_exact(&mut name_buf);
                let filename = String::from_utf8_lossy(&name_buf);

                println!("Receiving file: {}", filename);
                // 3. Read the file size (We sent a u64, which is 8 bytes)

                let mut size_buf = [0u8; 8];
                socket
                    .read_exact(&mut size_buf)
                    .expect("Failed to read size");
                let file_size = u64::from_be_bytes(size_buf);

                // 4. Create the file on disk
                // We use &*filename to tell Cow to act like a normal string
                let mut out_file = File::create(&*filename).expect("Failed to create file");

                // 5. The loop to actually save the bytes
                let mut buffer = [0u8; 8192];
                let mut received_so_far = 0;

                while received_so_far < file_size {
                    let bytes_read = socket.read(&mut buffer).expect("Network read error");
                    if bytes_read == 0 {
                        break;
                    } // Sender hung up

                    out_file
                        .write_all(&buffer[..bytes_read])
                        .expect("Disk write error");
                    received_so_far += bytes_read as u64;
                }

                println!("Portal: Transfer complete! Saved as {}", filename);
            }
        }
    }
}
