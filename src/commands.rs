use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::futures::TryFutureExt;
use serenity::model::prelude::*;
use serenity::prelude::Context;

use crate::DATABASE;
use crate::errors::{DiscordError, SipError};
use crate::logger::current_date_time;


#[command]
#[owners_only]
#[aliases("sip")]
pub async fn sip(context: &Context, message: &Message, _: Args) -> CommandResult {

    let channel_webhooks: Vec<Webhook> = message.channel_id.webhooks(&context.http).map_err(|err| DiscordError::DiscordWebhookError(err.to_string())).await?;
    let bot_id = context.cache.current_user_id();

    let mut current_webhook: Option<Webhook> = None;

    for webhook in channel_webhooks {

        match (&webhook.name, &webhook.user) {
            (Some(name), Some(user)) => {
                if name == "SipBot Webhook" && user.id == bot_id {
                    current_webhook = Some((&webhook).clone());
                    break;
                }
            },
            _ => {}
        }
    };

    let current_date_time: (String, String) = current_date_time();

    if let Some(existing_webhook) = current_webhook { // webhook postoji, brisemo ga
        let existing_webhook_id: WebhookId = existing_webhook.id;
        existing_webhook.delete(&context.http).map_err(|err| DiscordError::DiscordWebhookError(err.to_string())).await?;

        {
            let mut database = DATABASE.lock().await;
            let mut storage_webhooks: Vec<Webhook> = database.get::<Vec<Webhook>>("sip_hooks").unwrap_or(Vec::<Webhook>::new());
            storage_webhooks.retain(|storage_webhook| storage_webhook.id != existing_webhook_id);
            database.set("sip_hooks", &storage_webhooks).map_err(|err| SipError::StorageError(err.to_string()))?;
        }

        message.channel_id.send_message(&context.http, |m|
            m
                .embed(|e|
                    e
                        .author(|a| a.name("SIP").url("https://sip.elfak.ni.ac.rs/"))
                        .thumbnail("https://i.imgur.com/dyu12dZ.png")
                        .title(":warning: WebHook obrisan :warning:")
                        .color(0x65BD36)
                        .footer(|f|
                            f
                                .text(format!("Zahtevao {}#{} u {} dana {}", message.author.name, message.author.discriminator, current_date_time.1, current_date_time.0))
                                .icon_url(message.author.avatar_url().unwrap_or("https://i.imgur.com/dyu12dZ.png".to_string()))
                        )
                )
                .reference_message(message)
        ).map_err(|err| DiscordError::DiscordMessageError(err.to_string())).await?;

    } else { // webhook nije nadjen, pravimo ga i dodajemo u bazu
        current_webhook = Some(message.channel_id.create_webhook_with_avatar(&context.http, "SipBot Webhook", "https://i.imgur.com/dyu12dZ.png").map_err(|err| DiscordError::DiscordWebhookError(err.to_string())).await?);

        {
            let mut database = DATABASE.lock().await;
            let mut storage_webhooks: Vec<Webhook> = database.get::<Vec<Webhook>>("sip_hooks").unwrap_or(Vec::<Webhook>::new());
            storage_webhooks.push((&current_webhook.unwrap()).clone());
            database.set("sip_hooks", &storage_webhooks).map_err(|err| SipError::StorageError(err.to_string()))?;
        }

        message.channel_id.send_message(&context.http, |m|
            m
                .embed(|e|
                    e
                        .author(|a| a.name("SIP").url("https://sip.elfak.ni.ac.rs/"))
                        .thumbnail("https://i.imgur.com/dyu12dZ.png")
                        .title(":warning: WebHook registrovan :warning:")
                        .color(0x65BD36)
                        .footer(|f|
                            f
                                .text(format!("Zahtevao {}#{} u {} dana {}", message.author.name, message.author.discriminator, current_date_time.1, current_date_time.0))
                                .icon_url(message.author.avatar_url().unwrap_or("https://i.imgur.com/dyu12dZ.png".to_string()))
                        )
                )
                .reference_message(message)
        ).map_err(|err| DiscordError::DiscordMessageError(err.to_string())).await?;
    }

    return Ok(());
}