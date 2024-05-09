use sha2::{Digest, Sha256, Sha512};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{self, TcpStream}};
use anyhow::Result;

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
        Ok(Self { server: net::TcpStream::connect(address).await? })
    }

    pub async fn send_message(&mut self, message: ClientMessage) -> Result<()> {
        let message_as_str = serde_json::to_string(&message)?;
        //Send message lenght
        self.server.write_all(dbg!(&TryInto::<u32>::try_into(message_as_str.as_bytes().len())?.to_be_bytes())).await?;

        //Send actual message
        self.server.write_all(message_as_str.as_bytes()).await?;

        //Flush stream
        self.server.flush().await?;
        
        Ok(())
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct ClientInfromation {
    pub uuid: String,
    pub username: String,
    
    ///If the client was attempting a connection this field contains all the data used by the connection
    pub connection_request: Option<ConnectionRequest>,
}

impl ClientInfromation {
    pub fn new(uuid: String, username: String, connection_request: Option<ConnectionRequest>) -> Self {
        Self { uuid, username, connection_request }
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
        Self { inner_message, client_information }
    }
}