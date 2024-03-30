use discord::DiscordClient;

mod audio;
mod discord;

#[tokio::main]
async fn main() {
    let client = DiscordClient::new();
    client.connect_gateway().await;
}
