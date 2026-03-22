use std::{io::Read, net::TcpListener};

fn main() {
    // 1. Bind to the "Localhost" address at Port 7878
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Receiver: Portal open. Waiting for a connection on port 7878...");

    // 2. Wait for a single connection (this "blocks" the code)
    let (mut socket, addr) = listener.accept().unwrap();
    println!("Receiver: Connection established with {}!", addr);

    // 3. Read what the sender sent
    let mut buffer = [0; 1024]; // A small 1KB buffer to hold the message
    let bytes_read = socket.read(&mut buffer).unwrap();

    // 4. Convert the bytes to a string and print it
    let message = String::from_utf8_lossy(&buffer[..bytes_read]);
    println!("Receiver received: {}", message);
}

/* TcpListener::bind: "Claims" the port. If another app is using 7878, this will fail.
 ​accept(): This is the "Hook." The program stops and waits here. It returns the Socket (the connection) and the Address of who connected.
​ [0; 1024]: We created a manual buffer to catch the incoming bytes.
  The "Half-Full" Buffer: If a message is only 10 bytes, socket.read returns 10, but the buffer array is still 1024 long. So &buffer[..bytes_read] slice is very important, it tells Rust "Only look at the first 10 spots."
*/
