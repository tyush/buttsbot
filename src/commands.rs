use log::trace;
use poise::serenity_prelude as serenity;

type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, crate::Data, Error>;

#[poise::command(slash_command, prefix_command)]
pub async fn shut(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("no you").await?;
    if let Some(guild_id) = ctx.guild_id() {
        ctx.data()
            .guilds
            .lock()
            .await
            .butt_cooldowns
            .insert(guild_id, tokio::time::Instant::now());
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("there is no help for you now.").await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn butt(
    ctx: Context<'_>,
    #[description = "Message to buttify"] message: String,
) -> Result<(), Error> {
    if let Some(buttified) = crate::buttify::buttify_sentence(&message) {
        if buttified != "butt" {
            trace!("butted \"{}\" to \"{}\"", &message, buttified);
            ctx.say(buttified).await?;
        }
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn target(
    ctx: Context<'_>,
    #[description = "Target ID"] target_id: serenity::UserId,
) -> Result<(), Error> {
    if target_id.get() == 995608528234483713 {
        ctx.say("hah you thought").await?;
    } else {
        let _ = crate::TARGETING.write().await.insert(target_id);
        ctx.say("target locked").await?;
    }
    Ok(())
}
