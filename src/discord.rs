use std::{sync::OnceLock, time::Duration};

use serenity::{
    Client,
    all::{ChannelId, GatewayIntents},
};
use tokio::sync::mpsc;

use crate::Config;

// username, online
pub static SENDER: OnceLock<mpsc::UnboundedSender<BroadcastEvent>> = OnceLock::new();

pub enum BroadcastEvent {
    Online { user: String, resumed: bool },
    Offline { user: String, duration: Duration },
}

pub async fn start_discord() {
    let config = Config::get();

    if config.discord_token.is_empty() {
        println!("Token empty, discord client not started");
        return;
    }

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES;

    let mut client = Client::builder(&config.discord_token, intents)
        .await
        .expect("Err creating client");

    let (tx, mut rx) = mpsc::unbounded_channel();
    SENDER.set(tx).unwrap();
    let http = client.http.clone();

    tokio::task::spawn(async move {
        loop {
            let event = rx.recv().await.unwrap();

            match event {
                BroadcastEvent::Online { user, resumed } => {
                    for (channel, options) in config.broadcast_channels.iter() {
                        if !options.users.contains(&user) {
                            continue;
                        }

                        if resumed {
                            let _ = ChannelId::new(*channel)
                                .say(http.clone(), options.resumed.replace("{user}", &user))
                                .await;
                        } else {
                            let _ = ChannelId::new(*channel)
                                .say(http.clone(), options.online.replace("{user}", &user))
                                .await;
                        }
                    }
                }
                BroadcastEvent::Offline { user, duration } => {
                    let seconds = duration.as_secs() % 60;
                    let minutes = (duration.as_secs() / 60) % 60;
                    let hours = (duration.as_secs() / 60) / 60;
                    let duration = format!("{:0>2}:{:0>2}:{:0>2}", hours, minutes, seconds);
                    for (channel, options) in config.broadcast_channels.iter() {
                        if !options.users.contains(&user) {
                            continue;
                        }

                        let _ = ChannelId::new(*channel)
                            .say(
                                http.clone(),
                                options
                                    .offline
                                    .replace("{user}", &user)
                                    .replace("{duration}", &duration),
                            )
                            .await;
                    }
                }
            }
        }
    });
    tokio::task::spawn(async move {
        client.start().await.unwrap();
    });
}
