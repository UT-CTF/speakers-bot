use const_format::formatcp;
use reqwest::Client;

mod gateway;
mod payload;

const API: &str = formatcp!("https://discord.com/api/v{API_VERSION}");
const API_VERSION: u8 = 10;

pub(crate) struct DiscordClient(Client);

impl DiscordClient {
    pub(crate) fn new() -> Self {
        DiscordClient(
            Client::builder()
                .user_agent(formatcp!(
                    "DiscordBot (https://github.com/UT-CTF/speakers-bot, {})",
                    env!("CARGO_PKG_VERSION")
                ))
                .build()
                .unwrap(),
        )
    }
}
