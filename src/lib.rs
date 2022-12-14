#![feature(result_option_inspect)]
#![feature(let_chains)]

pub mod buttify;

use std::{collections::HashMap, mem};

use lazy_static::lazy_static;
use log::{error, info, trace};
use once_cell::sync::{Lazy, OnceCell};
use rand::random;
use serenity::{
    async_trait,
    http::CacheHttp,
    model::{
        channel::Message,
        gateway::Ready,
        id::GuildId,
        prelude::{ChannelId, EmojiId, Reaction, ReactionType, UserId},
    },
    prelude::*,
    utils::MessageBuilder,
};
use tokio::sync::{Mutex, RwLock};
use tokio::time::Instant;

use crate::buttify::buttify_sentence;

const REACT_BUFFER_SIZE: usize = 30;
lazy_static! {
    static ref REACTS: RwLock<Vec<ReactionType>> =
        RwLock::new(Vec::with_capacity(REACT_BUFFER_SIZE));
}

static TARGETING: RwLock<Option<UserId>> = RwLock::const_new(None);

async fn is_target(x: UserId, cache: impl CacheHttp) -> bool {
    if let Some(target) = *TARGETING.read().await {
        target == x
    } else {
        false
    }
}

pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    env_logger::init();

    let mut guard = REACTS.write().await;
    for i in vec![
        ReactionType::Custom {
            animated: false,
            id: EmojiId::from(1016490373712977932),
            name: Some("barbarian2".to_owned()),
        },
        ReactionType::Custom {
            animated: false,
            id: EmojiId::from(1016490373712977932),
            name: Some("barbarian2".to_owned()),
        },
        ReactionType::Custom {
            animated: false,
            id: EmojiId::from(1016490373712977932),
            name: Some("barbarian2".to_owned()),
        },
        ReactionType::Custom {
            animated: false,
            id: EmojiId::from(1016490373712977932),
            name: Some("barbarian2".to_owned()),
        },
        ReactionType::Custom {
            animated: false,
            id: EmojiId::from(1015434106793885747),
            name: Some("gregregation".to_owned()),
        },
    ] {
        guard.push(i);
    }
    mem::drop(guard);

    // stripping the err part makes the api
    // more convenient to use as an optional
    // value rather than a falliable operation
    let env = |s| std::env::var(s).ok();

    let token = env("BUTTSBOT_TOKEN")
        .expect("Requires a bot token set in BUTTSBOT_TOKEN environment variable!");
    // currently uses https://discord.com/oauth2/authorize?client_id=995608528234483713&permissions=68608&scope=bot
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let mut client = Client::builder(token, intents)
        .event_handler(Buttsbot {
            guilds: Mutex::new(ButtState {
                butt_cooldowns: HashMap::new(),
                prefix: HashMap::new(),
            }),
        })
        .await
        .expect("Failed created client.");

    Ok(client.start().await?)
}

pub struct Buttsbot {
    guilds: Mutex<ButtState>,
}

pub struct ButtState {
    butt_cooldowns: HashMap<GuildId, Instant>,
    prefix: HashMap<GuildId, String>,
}

#[async_trait]
impl EventHandler for Buttsbot {
    async fn message(&self, ctx: Context, msg: Message) {
        info!("{:?}", msg);
        if msg.author.id == UserId(173247397269471242) || msg.author.name.contains("pyratic") {
            trace!("potential command");
            let split = msg.content.split_once(' ');
            trace!("{:?}", split);
            if let Some((Ok(id), msg)) = split.map(|(id, m)| (id.parse::<u64>(), m)) {
                if let Some(ch) = ctx.cache.channel(ChannelId(id)) {
                    trace!("{:?}", ch);
                    let message = MessageBuilder::new().push(msg).build();
                    if let Err(e) = ch.id().say(&ctx, message).await {
                        error!("could not send admin message: {e}");
                    }
                }
            }
        }

        let prefix = {
            let lock = self.guilds.lock().await;
            msg.guild_id
                .and_then(|id| lock.prefix.get(&id))
                .map(String::clone)
                .unwrap_or("b!".to_owned())
        };

        let is_command = msg.content.starts_with(&prefix);

        const BUTT_CHANCE: f32 = 0.04;

        // function that adjusts chance of buttification
        // based on time since last buttification
        // https://www.desmos.com/calculator/ie4fbaqiby
        let a = 1.0717 * f32::powi(10., -11);
        let calc_chance = |time_since: f32| f32::min(a * time_since.powi(3), 0.03);

        if msg.is_own(&ctx.cache) {
            return;
        }

        if msg.author.id.0 == 1016490711929077780
            || msg.author.name.contains("Spenz")
            || msg
                .author_nick(&ctx)
                .await
                .map(|s| s.contains("Spenz"))
                .unwrap_or(false)
        {
            msg.react(
                &ctx,
                ReactionType::Custom {
                    animated: false,
                    id: EmojiId::from(1016490373712977932),
                    name: Some("barbarian2".to_owned()),
                },
            )
            .await
            .inspect_err(|e| error!("failed spencing: {}", e));

            msg.react(
                &ctx,
                REACTS
                    .read()
                    .await
                    .get((random::<f32>() * REACTS.read().await.len() as f32) as usize)
                    .expect("random of guard len not in vec")
                    .clone(),
            )
            .await
            .inspect_err(|e| error!("failed spencing: {}", e));
        }

        if msg.author.id.0 == 808891538591186954
            || msg.author.name.contains("GG1223")
            || msg
                .author_nick(&ctx)
                .await
                .map(|s| s.contains("GG1223"))
                .unwrap_or(false)
        {
            for e in [
                ReactionType::Unicode("\u{1F1F7}".to_string()),
                ReactionType::Unicode("\u{1F1E6}".to_string()),
                ReactionType::Unicode("\u{1F1F9}".to_string()),
                ReactionType::Unicode("\u{1F1EE}".to_string()),
                ReactionType::Unicode("\u{1F1F4}".to_string()),
            ] {
                msg.react(&ctx, e)
                    .await
                    .inspect_err(|e| error!("failed garing: {}", e));
            }
        }

        if is_command {
            trace!("received command {}", msg.content);
            // todo: add commands like prefix changing
            let without_prefix = &msg.content[prefix.len()..];

            if let Some(command) = without_prefix.split(" ").next() {
                match command {
                    "shut" => {
                        if let Err(e) = msg.reply(&ctx, "no you").await {
                            info!("failed replying: {}", e);
                        }
                        if let Some(guild) = msg.guild_id {
                            self.guilds
                                .lock()
                                .await
                                .butt_cooldowns
                                .insert(guild, Instant::now());
                        }
                    }
                    "help" => {
                        if let Err(e) = msg.reply(&ctx, "there is no help for you now.").await {
                            info!("failed replying: {}", e);
                        }
                    }
                    "butt" => {
                        let buttified = buttify_sentence(&without_prefix[4..]);
                        if let Some(buttified) = buttified {
                            if buttified != "butt" {
                                trace!("butted \"{}\" to \"{}\"", &msg.content, buttified);

                                if let Err(e) = msg.channel_id.say(&ctx, buttified).await {
                                    info!(
                                        "failed to send buttified message in guild {:?}: {}",
                                        msg.guild(&ctx.cache).map(|g| g.name),
                                        e
                                    );
                                }
                            }
                        }
                    }
                    "target" if !is_target(msg.author.id, &ctx).await => {
                        if let Some((_command, target_id)) = without_prefix.rsplit_once(" ")
                        && let Ok(x) = str::parse(target_id) {
                            if x == 995608528234483713 {
                                msg.reply(&ctx, "hah you thought").await.ok();
                            } else {

                            mem::drop(TARGETING.write().await.insert(x));
                            if let Err(e) = msg.channel_id.say(&ctx, "target locked").await {
                                info!("failed to send target lock message");
                            }
                            }
                        }
                    }
                    x => {
                        if let Err(e) = msg
                            .reply(
                                &ctx,
                                format!("tf are you on about? \"{}\" looking head-ass", x),
                            )
                            .await
                        {
                            info!("failed replying: {}", e);
                        }
                    }
                }
            }
        } else {
            if let Some(guild) = msg.guild_id {
                let mut guilds = self.guilds.lock().await;

                let mut butt_chance = guilds
                    .butt_cooldowns
                    .get(&guild)
                    .map(|last| calc_chance(Instant::now().duration_since(*last).as_secs_f32()))
                    .unwrap_or(BUTT_CHANCE);

                if is_target(msg.author.id, &ctx).await {
                    butt_chance = butt_chance.cbrt();
                    trace!("msg is from target");
                }

                trace!(
                    "butt chance of msg from {}: {:.2}",
                    &guild
                        .name(&ctx.cache)
                        .unwrap_or("[unknown guild]".to_string()),
                    butt_chance
                );

                if random::<f32>() < butt_chance {
                    let buttified = buttify_sentence(&msg.content);
                    if let Some(buttified) = buttified {
                        if buttified != "butt" {
                            trace!("butted \"{}\" to \"{}\"", &msg.content, buttified);

                            if let Err(e) = msg.reply(&ctx, buttified).await {
                                info!(
                                    "failed to send buttified message in guild {:?}: {}",
                                    msg.guild(&ctx.cache).map(|g| g.name),
                                    e
                                );
                            } else {
                                guilds.butt_cooldowns.insert(guild, Instant::now());
                            }
                        } else {
                            trace!(
                                "tried to buttify \"{}\", but just turned it into \"butt\"",
                                buttified
                            );
                        }
                    }
                }
            }
        }
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        let mut guard = REACTS.write().await;
        guard.insert(0, reaction.emoji.clone());
        guard.reverse();
        guard.truncate(REACT_BUFFER_SIZE);
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!(
            "Connected as {}#{:0>4}",
            ready.user.name, ready.user.discriminator
        );
    }
}

#[cfg(test)]
mod tests {}
