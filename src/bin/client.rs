use std::{fs, sync::Arc};
use tokio::{spawn, sync::Mutex};
use tokioplayground::{Client, ClientInfromation, ClientMessage};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Connecting to server. . .");

    let mut client =
        Client::new("[2a02:ab88:3713:8000:280a:a824:d7b2:7a52]:3000".to_string()).await?;

    loop {
        let mut input = String::new();

        println!("Connected, enter message:");

        std::io::stdin().read_line(&mut input)?;

        let input = input.trim().to_string();

        if input.is_empty() {
            //Disconnect client
            client.disconnect().await?;
            break;
        }

        client
            .send_message(ClientMessage::new(
                input,
                ClientInfromation::new("uuid".to_string(), "username".to_string(), None),
            ))
            .await?;

        let mut recv = client.reciver.resubscribe();

        spawn(async move {
            loop {
                for fasz in recv.recv().await {
                    dbg!(fasz);
                }
            }
        });
    }

    Ok(())
}
