use tokioplayground::{Client, ClientMessage, ClientInfromation};
use std::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = Client::new("[2a02:ab88:3713:8000:ccf0:2e3c:ac60:39a9]:3000".to_string()).await?;

    loop {
        client.send_message(ClientMessage::new(fs::read("C:\\Users\\marci\\Downloads\\Matthias.exe")?.to_vec().iter().map(|byte| {
            byte.to_string()
        }).collect() , ClientInfromation::new("uuid".to_string(), "username".to_string(), None))).await?;
    }
    
    Ok(())
}
