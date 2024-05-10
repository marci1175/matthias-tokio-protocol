use std::sync::Arc;

use anyhow::Result;
use sha2::{Digest, Sha256, Sha512};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{self, TcpStream},
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
    pub server: TcpStream,
}

impl Client {
    pub async fn new(address: String) -> Result<Self> {
        Ok(Self {
            server: net::TcpStream::connect(address).await?,
        })
    }

    pub async fn send_message(&mut self, message: ClientMessage) -> Result<()> {
        let message_as_str = serde_json::to_string(&message)?;

        //Send message lenght
        let message_lenght = TryInto::<u32>::try_into(message_as_str.as_bytes().len())?;

        self.server.write_all(&message_lenght.to_be_bytes()).await?;

        //Send actual message
        self.server.write_all(message_as_str.as_bytes()).await?;

        //Flush stream
        self.server.flush().await?;

        Ok(())
    }
}

// pub fn listen_for_messages(client: Arc<Mutex<Client>>) -> Receiver<String>  {
//     let (sender, reciver) = mpsc::channel::<String>(255);

//         let _: tokio::task::JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {

//             let mut client = client.lock().await;

//             loop {
//                 let mut message_len_buffer: Vec<u8> = vec![0; 4];

//                 client.server.read_exact(&mut message_len_buffer).await?;

//                 let incoming_message_len = u32::from_be_bytes(message_len_buffer[..4].try_into()?);

//                 let mut message_buffer: Vec<u8> = vec![0; incoming_message_len as usize];

//                 client.server.read_exact(&mut message_buffer).await?;

//                 //Send back message buffer
//                 sender.send(String::from_utf8(message_buffer)?).await?;
//             }

//             Ok(())
//         });

//         reciver
// }

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
