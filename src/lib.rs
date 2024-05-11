use std::sync::Arc;

use anyhow::Result;
use sha2::{Digest, Sha256, Sha512};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{self, tcp::{OwnedReadHalf, OwnedWriteHalf}, TcpStream},
    sync::{
        mpsc::{self, Receiver},
        Mutex,
    },
};

pub fn hash_string(num: String) -> [u8; 64] {
    let mut hasher = Sha512::new();

    hasher.update(num);

    let result = hasher.finalize();

    result.into()
}

pub struct Client {
    pub server_writer: OwnedWriteHalf,
    pub reciver: Receiver<String>,
}

impl Client {
    pub async fn new(address: String) -> Result<Self> {
        let (mut read, write) = net::TcpStream::connect(address).await?.into_split();

        let (sender, reciver) = mpsc::channel::<String>(255);

        let _: tokio::task::JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
            //aquire lock inside thread
            loop {
                //Peek messages, and if there are 4 bytes we can assume thats the lenght of the server's reply
                let mut message_len_buffer = [0; 4];

                let peeked_bytes = read.peek(&mut message_len_buffer).await?;

                //if peeked_bytes is 0 then it means the server hasnt replied, we should break the loop thus exit the thread
                if dbg!(peeked_bytes) == 0 {
                    break;
                }

                //Try to turn it into a u32
                let incoming_message_lenght = u32::from_be_bytes(message_len_buffer[..4].try_into()?);

                let mut message_buffer: Vec<u8> = vec![0; incoming_message_lenght as usize];

                read.read_exact(&mut message_buffer).await?;

                //drain message lenght cuz that was peeked already
                message_buffer.drain(0..4);

                //Send back to reciver
                sender.send(String::from_utf8(message_buffer)?).await?;
            }

            Ok(())
        });

        Ok(Self {
            server_writer: write,
            reciver: reciver,
        })
    }

    pub async fn send_message(&mut self, message: ClientMessage) -> Result<()> {
        let message_as_str = serde_json::to_string(&message)?;

        //Send message lenght
        let message_lenght = TryInto::<u32>::try_into(message_as_str.as_bytes().len())?;

        self.server_writer.write_all(&message_lenght.to_be_bytes()).await?;

        //Send actual message
        self.server_writer.write_all(message_as_str.as_bytes()).await?;

        //Flush stream
        self.server_writer.flush().await?;

        println!("MESSAGE SENT");

        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        //Send disconnect req to server
        self.server_writer.write_all(&u32::MAX.to_be_bytes()).await?;

        self.server_writer.shutdown().await?;

        Ok(())
    }

    // pub async fn try_recv_messages(&mut self) -> Result<Receiver<String>> {
        
    // }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct ClientInfromation {
    pub uuid: String,
    pub username: String,

    ///If the client was attempting a connection this field contains all the data used by the connection
    pub connection_request: Option<ConnectionRequest>,
}

impl ClientInfromation {
    pub fn new(
        uuid: String,
        username: String,
        connection_request: Option<ConnectionRequest>,
    ) -> Self {
        Self {
            uuid,
            username,
            connection_request,
        }
    }
}

///This struct contains all the information needed to connect to a server
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct ConnectionRequest {
    pub password: String,
}

///Main client message wrapper
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct ClientMessage {
    pub client_information: ClientInfromation,

    pub inner_message: String,
}

impl ClientMessage {
    pub fn new(inner_message: String, client_information: ClientInfromation) -> Self {
        Self {
            inner_message,
            client_information,
        }
    }
}
