use std::{fs, sync::Arc};
use tokio::sync::Mutex;
use tokioplayground::{Client, ClientInfromation, ClientMessage};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client =
        Client::new("[2a02:ab88:3713:8000:a5c1:fc5a:b1e:a283]:3000".to_string()).await?;

    //start listening thread
    // let mut incoming_messages = listen_for_messages(client.clone());

    loop {
        let mut input = String::new();

        std::io::stdin().read_line(&mut input)?;
        
        client
            .send_message(ClientMessage::new(
                input,
                ClientInfromation::new("uuid".to_string(), "username".to_string(), None),
            ))
            .await?;
    }

    Ok(())
}
