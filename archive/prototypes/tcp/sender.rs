use std::{
net::TcpStream,
io::Write
}; // since we are sending

fn main() {
    println!("Sender: Attempting to connect to the portal...");

    // 1. Connect to the Receiver (This triggers the 3-way handshake)
    let mut stream = TcpStream::connect("127.0.0.1:7878").expect("Could not connect!");
    println!("Sender: Connected!");

    // 2. Prepare a message to send
    let message = "Hello from the Hiverra Sender!";
    
    // 3. Write the message to the stream (Pushing bytes into the pipe)
    stream.write_all(message.as_bytes()).unwrap();
    println!("Sender: Message sent. Closing connection.");
}


/*  unwrap(): The "Quick and Dirty" way to get a value out of a Result. It's great for testing but dangerous for production because it causes a panic.
​Automatic Closing: Rust handles the "hang up" for you. You don't usually need a stream.close() command; the connection dies the moment the variable is dropped.
​The Buffer: The [0; 1024] syntax means "Create an array with 1024 elements, and set every single one of them to zero."
*/