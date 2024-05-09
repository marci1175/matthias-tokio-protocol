use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{self, tcp}};
use tokioplayground::ClientMessage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let tcp_listener = net::TcpListener::bind("[::]:3000").await?;

    loop {
        let mut message_len_buffer: Vec<u8> = vec![0; 4];

        let (mut stream, address) = tcp_listener.accept().await?;

        stream.readable().await?;

        stream.read_exact(&mut message_len_buffer).await?;

        let incoming_message_len = u32::from_be_bytes(message_len_buffer[..4].try_into()?);

        let mut message_buffer: Vec<u8> = vec![0; incoming_message_len as usize];

        stream.read_exact(&mut message_buffer).await?;

        let message = String::from_utf8(message_buffer)?;

        // let parsed_message: ClientMessage = serde_json::from_str(&message.trim())?;

        dbg!(&message, message.len(), incoming_message_len);

        //Flush stream
        stream.flush().await?;

        stream.write_all(b"Reply").await?;
    }
}
