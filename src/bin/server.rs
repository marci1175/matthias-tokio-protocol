use std::{net::SocketAddr, sync::Arc};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{self, tcp, TcpStream},
    sync::Mutex,
};
use tokioplayground::ClientMessage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let tcp_listener = net::TcpListener::bind("[::]:3000").await?;

    //Information which should be stored simulating a server
    let messages: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let connected_clients: Arc<Mutex<Vec<SocketAddr>>> = Arc::new(Mutex::new(Vec::new()));

    loop {
        //move arc mutex
        let messages_clone = messages.clone();
        let connected_clients_clone = connected_clients.clone();

        let (mut stream, address) = tcp_listener.accept().await?;

        //Push into client list
        connected_clients_clone.lock().await.push(address);

        let _: tokio::task::JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
            //move address which we are connected to
            let connected_address = address.clone();

            loop {
                stream.readable().await?;

                let mut message_len_buffer: Vec<u8> = vec![0; 4];

                stream.read_exact(&mut message_len_buffer).await?;

                let incoming_message_len = u32::from_be_bytes(message_len_buffer[..4].try_into()?);

                let mut message_buffer: Vec<u8> = vec![0; incoming_message_len as usize];

                stream.read_exact(&mut message_buffer).await?;

                let message = String::from_utf8(message_buffer)?;

                let parsed_message: ClientMessage = serde_json::from_str(&message.trim())?;

                //store message
                messages_clone
                    .lock()
                    .await
                    .push(parsed_message.inner_message);

                dbg!(&messages_clone);

                let mut peeking_bytes = [0; 4];

                stream.peek(&mut peeking_bytes).await?;

                //If we sent u32::MAX that means we want to disconnect
                if u32::from_be_bytes(peeking_bytes[..4].try_into()?) == u32::MAX {
                    stream.shutdown().await?;
                    
                    //Remove from client list
                    let mut connected_clients = connected_clients_clone.lock().await;

                    //search through vector
                    if let Some(index) = connected_clients.iter().position(|p| *p == connected_address) {
                        connected_clients.remove(index);
                    }

                    break;
                }
            }
            Ok(())
        });

        //Thread end
        println!("Connected clients: ");
        dbg!(connected_clients.lock().await);
    }
}
