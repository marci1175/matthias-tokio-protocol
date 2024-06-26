use tokio::spawn;
use tokioplayground::Client;

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

        client.send_message(String::from("asd")).await?;

        let mut recv = client.reciver.resubscribe();

        //Recive messages
        spawn(async move {
            loop {
                while let Ok(fasz) = recv.recv().await {
                    dbg!(fasz);
                }
            }
        });
    }

    Ok(())
}
