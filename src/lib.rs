use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        self,
        tcp::OwnedWriteHalf,
    },
    sync::broadcast,
};

pub struct Client {
    pub server_writer: OwnedWriteHalf,
    pub reciver: tokio::sync::broadcast::Receiver<String>,
}

impl Client {
    pub async fn new(address: String) -> Result<Self> {
        let (mut read, write) = net::TcpStream::connect(address).await?.into_split();

        let (sender, reciver) = broadcast::channel::<String>(255);

        let _: tokio::task::JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
            //aquire lock inside thread
            loop {
                //Wait until message lenght arrives
                read.readable().await?;

                //Peek messages, and if there are 4 bytes we can assume thats the lenght of the server's reply
                let mut message_len_buffer = [0; 4];

                let peeked_bytes = read.read(&mut message_len_buffer).await?;

                //if peeked_bytes is 0 then it means the server hasnt replied, we should break the loop thus exit the thread
                if peeked_bytes != 4 {
                    break;
                }

                //Try to turn it into a u32
                let incoming_message_lenght =
                    u32::from_be_bytes(message_len_buffer[..4].try_into()?);

                let mut message_buffer: Vec<u8> = vec![0; incoming_message_lenght as usize];

                //Check if server has sent the main message
                read.readable().await?;

                read.read_exact(&mut message_buffer).await?;

                //Send back to reciver
                sender.send(String::from_utf8(message_buffer)?)?;
            }

            Ok(())
        });

        Ok(Self {
            server_writer: write,
            reciver: reciver,
        })
    }

    pub async fn send_message(&mut self, message: String) -> Result<()> {
        //Send message lenght
        let message_lenght = TryInto::<u32>::try_into(message.as_bytes().len())?;

        self.server_writer
            .write_all(&message_lenght.to_be_bytes())
            .await?;

        //Send actual message
        self.server_writer.write_all(message.as_bytes()).await?;

        //Flush stream
        self.server_writer.flush().await?;

        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        //Send disconnect req to server
        self.server_writer
            .write_all(&u32::MIN.to_be_bytes())
            .await?;

        self.server_writer.shutdown().await?;

        Ok(())
    }

    pub async fn clone_reciver(&self) -> tokio::sync::broadcast::Receiver<String> {
        self.reciver.resubscribe()
    }
}