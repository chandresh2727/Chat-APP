use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::collections::HashSet;
use std::sync::Arc;
use std::thread;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let (tx, _rx) = broadcast::channel(10);
    let clients = Arc::new(tokio::sync::Mutex::new(HashSet::new()));

    println!("Server listening on port 8080...");

    while let Ok((mut socket, _)) = listener.accept().await {
        let tx = tx.clone();
        let clients = clients.clone();

        tokio::spawn(handle_client(socket, tx, clients));
    }

    Ok(())
}

async fn handle_client(
    mut socket: TcpStream,
    tx: broadcast::Sender<String>,
    clients: Arc<tokio::sync::Mutex<HashSet<String>>>,
) {
    let (mut reader, mut writer) = socket.split();
    let mut client_name = String::new();

    // Read the client's name
    if let Err(_) = reader.read_to_string(&mut client_name).await {
        return;
    }

    // Add the client's name to the set of clients
    let mut clients = clients.lock().await;
    clients.insert(client_name.clone());

    // Notify all clients about the new connection
    tx.send(format!("{} joined the chat.", client_name)).ok();

    // Broadcast messages to all connected clients
    let mut rx = tx.subscribe();
    while let Some(message) = rx.recv().await.ok() {
        if writer.write_all(message.as_bytes()).await.is_err() {
            break;
        }
    }

    // Remove the client's name from the set of clients
    clients.remove(&client_name);
    tx.send(format!("{} left the chat.", client_name)).ok();
}

