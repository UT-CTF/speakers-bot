use std::env::consts::OS;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use crate::audio::play_doorbell;
use const_format::formatcp;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_repr::Serialize_repr;
use tokio::spawn;
use tokio::sync::mpsc::channel;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use url::Url;

use crate::discord::payload::{Opcode, Payload};

use super::DiscordClient;
use super::API;

impl DiscordClient {
    pub(crate) async fn connect_gateway(&self) {
        let url = self.get_gateway_url().await;

        loop {
            let socket = connect_async(Url::parse(url.deref()).unwrap())
                .await
                .unwrap()
                .0;

            let (mut output_stream, mut input) = socket.split();
            let (output, mut receiver) = channel::<Payload>(1);
            spawn(async move {
                while let Some(payload) = receiver.recv().await {
                    output_stream
                        .send(Message::text(
                            dbg!(serde_json::to_string(&payload).unwrap()),
                        ))
                        .await
                        .unwrap();
                }
            });

            let s: Arc<Mutex<Option<u64>>> = Arc::new(Mutex::new(None));

            while let Some(Ok(message)) = input.next().await {
                let payload: Payload =
                    serde_json::from_str(dbg!(message.into_text().unwrap().as_str())).unwrap();
                match payload.op {
                    Opcode::Dispatch => {
                        s.lock().await.replace(payload.s.unwrap());
                        if let Some(event_name) = payload.t {
                            if event_name.deref() == "INTERACTION_CREATE" {
                                #[derive(Deserialize)]
                                struct Interaction {
                                    id: Box<str>,
                                    token: Box<str>,
                                }

                                let interaction: Interaction =
                                    serde_json::from_value(payload.d).unwrap();

                                #[derive(Serialize)]
                                struct Response {
                                    #[serde(rename = "type")]
                                    callback_type: CallbackType,
                                    data: CallbackData,
                                }

                                #[repr(u8)]
                                #[derive(Serialize_repr)]
                                enum CallbackType {
                                    ChannelMessageWithSource = 4,
                                }

                                #[derive(Serialize)]
                                struct CallbackData {
                                    content: &'static str,
                                    flags: u64,
                                }

                                self.0
                                    .post(format!(
                                        "{API}/interactions/{}/{}/callback",
                                        interaction.id, interaction.token
                                    ))
                                    .json(&Response {
                                        callback_type: CallbackType::ChannelMessageWithSource,
                                        data: CallbackData {
                                            content: "The Doorbell has been Rung!",
                                            flags: 1 << 6,
                                        },
                                    })
                                    .send()
                                    .await
                                    .unwrap();

                                play_doorbell().await;
                            }
                        }
                    }
                    Opcode::Heartbeat => {
                        output
                            .send(Payload {
                                op: Opcode::Heartbeat,
                                d: serde_json::to_value(s.lock().await.deref()).unwrap(),
                                s: None,
                                t: None,
                            })
                            .await
                            .unwrap();
                    }
                    Opcode::PresenceUpdate => {}
                    Opcode::Reconnect => {
                        break;
                    }
                    Opcode::InvalidSession => {
                        // TODO Resume?
                        break;
                    }
                    Opcode::Hello => {
                        #[derive(Deserialize)]
                        struct Hello {
                            heartbeat_interval: u16,
                        }
                        let interval = serde_json::from_value::<Hello>(payload.d)
                            .unwrap()
                            .heartbeat_interval;

                        #[derive(Serialize)]
                        struct Identify {
                            token: &'static str,
                            properties: ConnectionProperties,
                            intents: u8,
                        }
                        #[derive(Serialize)]
                        struct ConnectionProperties {
                            os: &'static str,
                            browser: &'static str,
                            device: &'static str,
                        }
                        const PKG_NAME: &str = env!("CARGO_PKG_NAME");
                        output
                            .send(Payload {
                                op: Opcode::Identify,
                                d: serde_json::to_value(&Identify {
                                    token: env!("SPEAKERS_BOT_TOKEN"),
                                    properties: ConnectionProperties {
                                        os: OS,
                                        browser: PKG_NAME,
                                        device: PKG_NAME,
                                    },
                                    intents: 0,
                                })
                                .unwrap(),
                                s: None,
                                t: None,
                            })
                            .await
                            .unwrap();

                        let output = output.clone();
                        let s = s.clone();
                        spawn(async move {
                            loop {
                                sleep(Duration::from_millis(interval as u64)).await;
                                output
                                    .send(Payload {
                                        op: Opcode::Heartbeat,
                                        d: serde_json::to_value(s.lock().await.deref()).unwrap(),
                                        s: None,
                                        t: None,
                                    })
                                    .await
                                    .unwrap();
                            }
                        });
                    }
                    Opcode::HeartbeatACK => {}
                    _ => unreachable!(),
                }
            }
        }
    }

    async fn get_gateway_url(&self) -> Box<str> {
        #[derive(Deserialize)]
        struct Response {
            url: Box<str>,
        }

        self.0
            .get(formatcp!("{API}/gateway"))
            .send()
            .await
            .unwrap()
            .json::<Response>()
            .await
            .unwrap()
            .url
    }
}
