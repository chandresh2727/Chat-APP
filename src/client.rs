use tokio::net::TcpStream;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <username>", args[0]);
        std::process::exit(1);
    }
    let username = &args[1];

    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    let (reader, mut writer) = io::split(&mut stream);

    // Send the username to the server
    writer.write_all(username.as_bytes()).await?;
    writer.flush().await?;

    let mut reader = io::BufReader::new(reader);
    let mut buffer = String::new();

    // Read and display messages from the server
    tokio::spawn(async move {
        loop {
            match reader.read_line(&mut buffer).await {
                Ok(0) => {
                    println!("Server disconnected.");
                    break;
                }
                Ok(_) => {
                    println!("{}", buffer);
                    buffer.clear();
                }
                Err(_) => {
                    println!("Error reading from server.");
                    break;
                }
            }
        }
    });

    // Read user input and send messages to the server
    let mut user_input = String::new();
    loop {
        io::stdin().read_line(&mut user_input).await?; // No need to use .lock() here
        writer.write_all(user_input.as_bytes()).await?;
        writer.flush().await?;
        user_input.clear();
    }

    Ok(())
}
