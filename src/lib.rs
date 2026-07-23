pub mod bot_ai;
pub mod buttify;
pub mod commands;

use std::collections::HashMap;
use std::mem;

use lazy_static::lazy_static;
use log::{error, info, trace};
use rand::random;
use serenity::{
    all::{EmojiId, ReactionType},
    http::CacheHttp,
    model::{
        id::GuildId,
        prelude::{ChannelId, UserId},
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

pub static TARGETING: RwLock<Option<UserId>> = RwLock::const_new(None);

pub async fn is_target(x: UserId, _cache: impl CacheHttp) -> bool {
    if let Some(target) = *TARGETING.read().await {
        target == x
    } else {
        false
    }
}

pub struct Data {
    pub guilds: Mutex<ButtState>,
    pub ai: Mutex<bot_ai::AiBot>,
}

pub struct ButtState {
    pub butt_cooldowns: HashMap<GuildId, Instant>,
    pub prefix: HashMap<GuildId, String>,
}

pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    env_logger::init();

    let mut guard = REACTS.write().await;
    for i in [
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

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::shut(),
                commands::help(),
                commands::butt(),
                commands::target(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("b!".into()),
                ..Default::default()
            },
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                info!("Initializing AI model...");
                let ai_bot = match bot_ai::AiBot::new().await {
                    Ok(bot) => {
                        info!("AI model loaded successfully");
                        bot
                    },
                    Err(e) => {
                        error!("Failed to load AI model: {}. Bot will still run but AI features will fail.", e);
                        // We could either crash here or return a bot that panics on generation,
                        // but crashing early is usually better if we explicitly want this feature.
                        return Err(e.into());
                    }
                };

                Ok(Data {
                    guilds: Mutex::new(ButtState {
                        butt_cooldowns: HashMap::new(),
                        prefix: HashMap::new(),
                    }),
                    ai: Mutex::new(ai_bot),
                })
            })
        })
        .build();

    let mut client = Client::builder(token, intents)
        .framework(framework)
        .await
        .expect("Failed creating client.");

    Ok(client.start().await?)
}

async fn event_handler(
    ctx: &Context,
    event: &serenity::all::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Box<dyn std::error::Error + Send + Sync>>,
    data: &Data,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match event {
        serenity::all::FullEvent::Ready { data_about_bot } => {
            info!("Connected as {}", data_about_bot.user.name);
        }
        serenity::all::FullEvent::ReactionAdd { add_reaction } => {
            let mut guard = REACTS.write().await;
            guard.insert(0, add_reaction.emoji.clone());
            guard.reverse();
            guard.truncate(REACT_BUFFER_SIZE);
        }
        serenity::all::FullEvent::Message { new_message: msg } => {
            info!("{:?}", msg);
            if msg.author.id == UserId::new(173247397269471242)
                || msg.author.name.contains("pyratic")
            {
                trace!("potential command");
                let split = msg.content.split_once(' ');
                trace!("{:?}", split);
                if let Some((Ok(id), msg_content)) = split.map(|(id, m)| (id.parse::<u64>(), m)) {
                    let channel_id = ChannelId::new(id);
                    let message = MessageBuilder::new().push(msg_content).build();
                    if let Err(e) = channel_id.say(&ctx, message).await {
                        error!("could not send admin message: {e}");
                    }
                }
            }

            const BUTT_CHANCE: f32 = 0.04;

            let a = 1.0717 * f32::powi(10., -11);
            let calc_chance = |time_since: f32| f32::min(a * time_since.powi(3), 0.03);

            if msg.author.id == ctx.cache.current_user().id {
                return Ok(());
            }

            let bot_name = ctx.cache.current_user().name.clone();
            if msg.content.contains(&bot_name) {
                // Remove the bot name from the prompt to avoid it talking about itself
                let prompt = msg.content.replace(&bot_name, "").trim().to_string();
                if !prompt.is_empty() {
                    let mut ai = data.ai.lock().await;
                    let reply_text = tokio::task::block_in_place(|| match ai.generate(prompt) {
                        Ok(text) => text,
                        Err(e) => {
                            error!("AI generation failed: {}", e);
                            "Oops, my brain farted.".to_string()
                        }
                    });

                    if let Err(e) = msg.reply(&ctx, reply_text).await {
                        error!("failed to reply with AI text: {}", e);
                    }
                }
            }

            if msg.author.id.get() == 1016490711929077780
                || msg.author.name.contains("Spenz")
                || msg
                    .author_nick(&ctx)
                    .await
                    .map(|s| s.contains("Spenz"))
                    .unwrap_or(false)
            {
                let _ = msg
                    .react(
                        &ctx,
                        ReactionType::Custom {
                            animated: false,
                            id: EmojiId::from(1016490373712977932),
                            name: Some("barbarian2".to_owned()),
                        },
                    )
                    .await
                    .inspect_err(|e| error!("failed spencing: {}", e));

                let _ = msg
                    .react(
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

            if msg.author.id.get() == 808891538591186954
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
                    let _ = msg
                        .react(&ctx, e)
                        .await
                        .inspect_err(|e| error!("failed garing: {}", e));
                }
            }

            if let Some(guild) = msg.guild_id {
                let mut guilds = data.guilds.lock().await;

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
                                    msg.guild(&ctx.cache).map(|g| g.name.clone()),
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
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {}
