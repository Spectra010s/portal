use {tokio::net::TcpListener,
 std::path::PathBuf,
 anyhow::Result,
 };
 mod receiver;


#[tokio::main]
async fn main() -> Result<()> {
    // Bind to the address
    let listener = TcpListener::bind("127.0.0.1:7878").await?;
    let target_dir = PathBuf::from("./received_dir");

    println!("Receiver: Waiting for sender...");

    // Accept connection from sender 
    let (socket, _) = listener.accept().await?;
    println!("Receiver: Connection established!");

    // Receive the directory
    receiver::receive_directory(socket, &target_dir).await?;

    println!("Receiver: Done receiving directory!");
    Ok(())
}