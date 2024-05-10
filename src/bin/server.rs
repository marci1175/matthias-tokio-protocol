use std::sync::Arc;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{self, tcp, TcpStream}, sync::Mutex,
};
use tokioplayground::ClientMessage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let tcp_listener = net::TcpListener::bind("[::]:3000").await?;

    //Information which should be stored simulating a server
    let mut messages: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let mut clients: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));

    loop {

        let mut messages_clone = messages.clone();
        let mut clients_clone = clients.clone();

        let (mut stream, address) = tcp_listener.accept().await?;

        let _: tokio::task::JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
            stream.readable().await?;

            let mut message_len_buffer: Vec<u8> = vec![0; 4];

            stream.read_exact(&mut message_len_buffer).await?;

            let incoming_message_len = u32::from_be_bytes(message_len_buffer[..4].try_into()?);

            let mut message_buffer: Vec<u8> = vec![0; incoming_message_len as usize];

            stream.read_exact(&mut message_buffer).await?;

            let message = String::from_utf8(message_buffer)?;

            let parsed_message: ClientMessage = serde_json::from_str(&message)?;

            //Flush stream
            stream.flush().await?;

            //store message
            messages_clone.lock().await.push(parsed_message.inner_message);
            // clients_clone.lock().await.push(stream);

            // for client in clients_clone.lock().await.iter_mut() {
            //     for message in &*messages_clone.lock().await {
            //         //Send message lenght
            //         let message_lenght = TryInto::<u32>::try_into(message.as_bytes().len())?;

            //         client.write_all(&message_lenght.to_be_bytes()).await?;

            //         //Send actual message
            //         client.write_all(message.as_bytes()).await?;

            //         //Flush stream
            //         client.flush().await?;
            //     }
            // }

            Ok(())
        });

        dbg!(&messages);

    }
}
