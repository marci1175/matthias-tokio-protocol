use std::{net::SocketAddr, sync::Arc, time::Duration};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        self,
        tcp::{self, OwnedWriteHalf},
        TcpStream,
    },
    sync::Mutex,
};
use tokioplayground::ClientMessage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let tcp_listener = net::TcpListener::bind("[::]:3000").await?;

    //Information which should be stored simulating a server
    let messages: Arc<Mutex<Vec<ClientMessage>>> = Arc::new(Mutex::new(Vec::new()));

    /* For future reference, we push back the ```OwnedWriteHalf``` of the client, and in the reader thread we should only be accessing it by ```connected_clients[self_id]``` */
    let connected_clients: Arc<Mutex<Vec<OwnedWriteHalf>>> = Arc::new(Mutex::new(Vec::new()));

    loop {
        //move arc mutex
        let messages_clone = messages.clone();
        let connected_clients_clone = connected_clients.clone();

        let (stream, _address) = tcp_listener.accept().await?;

        let (mut reader, writer) = stream.into_split();

        //Push into client list
        connected_clients_clone.lock().await.push(writer);

        //Reader thread
        let _: tokio::task::JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
            //This represents the clients place in the connected_clients list
            let self_id = connected_clients_clone.lock().await.len() - 1;

            loop {
                reader.readable().await?;

                let mut message_len_buffer: Vec<u8> = vec![0; 4];

                reader.read_exact(&mut message_len_buffer).await?;

                let incoming_message_len = u32::from_be_bytes(message_len_buffer[..4].try_into()?);

                //If we sent u32::MIN that means we want to disconnect
                if incoming_message_len == u32::MIN {
                    //Remove form list
                    connected_clients_clone.lock().await.remove(self_id);

                    break;
                }

                let mut message_buffer: Vec<u8> = vec![0; incoming_message_len as usize];

                //Wait until the client sends the main message
                reader.readable().await?;

                reader.read_exact(&mut message_buffer).await?;

                let message = String::from_utf8(message_buffer)?;

                let parsed_message: ClientMessage = serde_json::from_str(&message.trim())?;

                //store message
                {
                    messages_clone.lock().await.push(parsed_message);
                }

                //Reply to all clients after incoming msg
                reply_to_all_clients(connected_clients_clone.clone(), messages_clone.clone()).await?;
            }
            Ok(())
        });

        //// Clone again because of moved value
        // let connected_clients_clone = connected_clients.clone();
        // let messages_clone = messages.clone();
        //// Sync thread (Not needed)
        // sync_thread.get_or_insert_with(|| {
        //     tokio::spawn(async move {
        //         loop {
        //             //Thread sleep
        //             tokio::time::sleep(Duration::from_secs(3)).await;
        //             //Clone again because of moved value
        //             reply_to_all_clients(connected_clients_clone.clone(), messages_clone.clone()).await?;
        //         }
        //         Ok(())
        //     })
        // });
    }
}

/// This function iterates over all the connected clients and all the messages, and sends writes them all to their designated ```OwnedWriteHalf``` (All of the users see all of the messages)
pub async fn reply_to_all_clients(connected_clients_clone: Arc<Mutex<Vec<OwnedWriteHalf>>>, messages_clone: Arc<Mutex<Vec<ClientMessage>>>) -> anyhow::Result<()> {
    //Sleep thread
    let mut connected_clients = connected_clients_clone.lock().await;

    for client in connected_clients.iter_mut() {
        for message in messages_clone.lock().await.iter() {
            let message_as_str = serde_json::to_string(&message)?;

            //Send message lenght
            let message_lenght =
                TryInto::<u32>::try_into(message_as_str.as_bytes().len())?;

            client.write_all(&message_lenght.to_be_bytes()).await?;

            //Send actual message
            client.write_all(message_as_str.as_bytes()).await?;

            client.flush().await?;
        }
    }

    Ok(())
}

/// Implement server messages so we dont send out client messages to clients we should only be reciving them (we need to trim data connected to security for obv reasons)
pub struct ServerMessage {}