use std::{
    fs::File,
    io::{Read, Write},
    net::TcpListener,
};

pub fn receive_file() {
    println!("Portal: Initializing  systems...");
    let listener = TcpListener::bind("127.0.0.1:7878").expect("Failed to bind to port 7878");

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
