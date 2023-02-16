use std::{collections::HashSet};

use errors::DiscordError;
use pickledb::PickleDb;

use serenity::Client;
use serenity::http::{Http};
use serenity::futures::TryFutureExt;
use serenity::model::prelude::{CurrentApplicationInfo, UserId, Ready};
use serenity::framework::{StandardFramework};
use serenity::framework::standard::macros::group;
use serenity::prelude::{GatewayIntents, Context, EventHandler};

mod fetcher;
mod storage;
mod errors;
mod logger;
mod commands;
use commands::*;

use dotenv;

use tokio::sync::Mutex;

use crate::fetcher::fetcher_main;

const FILE: &str = "storage.json";

lazy_static::lazy_static! {
    // napravi bazu, ako ne postoji
    static ref DATABASE: Mutex<PickleDb> = Mutex::<PickleDb>::new(
        match PickleDb::load(FILE, pickledb::PickleDbDumpPolicy::AutoDump, pickledb::SerializationMethod::Json) {
            Err(_) => PickleDb::new(FILE, pickledb::PickleDbDumpPolicy::AutoDump, pickledb::SerializationMethod::Json),
            Ok(database) => database
        }
    );
}

#[group]
#[commands(sip)]
struct General;

struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, context: Context, _: Ready) {
        use serenity::model::gateway::Activity;
        use serenity::model::user::OnlineStatus;

        let activity = Activity::playing("https://cortex.inicijativa.software");
        let status = OnlineStatus::Online;

        context.set_presence(Some(activity), status).await;

        logger::log("READY", "SipBot is ready").await;
    }

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    dotenv::dotenv().ok();
    let token: String = std::env::var("TOKEN").expect("Missing TOKEN");
    let prefix: String = std::env::var("PREFIX").expect("Missing PREFIX");

    {
        let mut database = DATABASE.lock().await;
        storage::setup_storage(&mut database)?;
    }

    // discord bot
    let http: Http = Http::new(&token);

    let application_info: CurrentApplicationInfo = http.get_current_application_info().map_err(|err| DiscordError::DiscordAppInfoError(err.to_string())).await?;
    let mut owners: HashSet<UserId> = HashSet::<UserId>::new();
    owners.insert(application_info.owner.id);

    let framework = StandardFramework::new().configure(|c|
        c
            .allow_dm(false)
            .case_insensitivity(true)
            .ignore_bots(true)
            .with_whitespace(true)
            .prefix(prefix)
            .owners(owners)
    ).group(&GENERAL_GROUP);

    let intents: GatewayIntents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_WEBHOOKS | GatewayIntents::MESSAGE_CONTENT;
    let mut client: Client = Client::builder(&token, intents)
                        .event_handler(Handler)
                        .framework(framework)
                        .map_err(|err| DiscordError::DiscordBuilderError(err.to_string()))
                        .await?;

    let http_fetcher: Http = Http::new(&token);


    // pokrecemo dva nezavisna zadatka
    // discord bot i sip fetcher
    let (bot_handle, fetcher_handle) = tokio::join!(client.start(), tokio::task::spawn(async {
        fetcher_main(http_fetcher).await;
    }));

    dbg!(&bot_handle, &fetcher_handle);
    return Ok(());
}