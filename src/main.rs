use dotenv::dotenv;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::sync::Arc;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use types::*;

mod database;
mod inline;
mod messages;
mod types;
mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    pretty_env_logger::init();

    log::info!("Starting stimkerbot");

    let db = get_db().await?;

    let bot = Bot::from_env().parse_mode(teloxide::types::ParseMode::Html);

    let command_handler =
        dptree::filter(|msg: Message| msg.text().map(|t| t.starts_with("/")).unwrap_or(false))
            .endpoint(messages::command_handler);

    let message_recieve_qsimport_tree =
        dptree::case![ConversationState::ReceiveQSImport].endpoint(messages::receive_qs_import);

    let message_recieve_botimport_tree =
        dptree::case![ConversationState::ReceiveBotImport].endpoint(messages::receive_bot_import);

    let message_verify_stop_tree =
        dptree::case![ConversationState::VerifyStop].endpoint(messages::verify_stop);

    let message_receive_entities_ids_tree = dptree::case![ConversationState::RecieveEntitiesId]
        .endpoint(messages::receive_entities_ids);

    let message_receive_entities_tags_tree =
        dptree::case![ConversationState::RecieveEntitiesTags { entities }]
            .endpoint(messages::receive_entities_tags);

    let message_receive_entity_id_tree =
        dptree::case![ConversationState::ReceiveEntityId].endpoint(messages::receive_entity_id);

    let message_receive_entity_tags_tree = dptree::case![ConversationState::ReceiveEntityTags {
        entity,
        entity_type
    }]
    .endpoint(messages::receive_entity_tags);

    let message_tree = Update::filter_message()
        .enter_dialogue::<Message, InMemStorage<ConversationState>, ConversationState>()
        .branch(command_handler)
        .branch(message_recieve_qsimport_tree)
        .branch(message_recieve_botimport_tree)
        .branch(message_verify_stop_tree)
        .branch(message_receive_entities_ids_tree)
        .branch(message_receive_entities_tags_tree)
        .branch(message_receive_entity_id_tree)
        .branch(message_receive_entity_tags_tree);

    let inline_tree = Update::filter_inline_query().endpoint({
        let db = db.clone();
        move |bot, query| inline::handler_inline_query(db.clone(), bot, query)
    });

    let inline_result_tree = Update::filter_chosen_inline_result().endpoint({
        let db = db.clone();
        move |query| inline::handle_inline_choice(db.clone(), query)
    });

    let tree = dptree::entry()
        .branch(message_tree)
        .branch(inline_tree)
        .branch(inline_result_tree);

    log::debug!("Sending commands");

    bot.set_my_commands(messages::Command::bot_commands())
        .await?;

    log::debug!("Starting dispatcher");

    Dispatcher::builder(bot, tree)
        .dependencies(dptree::deps![InMemStorage::<ConversationState>::new(), db])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    log::debug!("Dispatcher stopped");

    Ok(())
}

async fn get_db() -> Result<DbType, Box<dyn std::error::Error>> {
    let database_location =
        std::env::var("DATABASE_LOCATION").expect("DATABASE_LOCATION must be set");
    log::debug!("Database location: {:?}", database_location);

    log::debug!("Opening/creating and migrating database");

    touch(database_location.clone());
    let db: DbType = Arc::new(
        SqlitePoolOptions::new()
            .connect_with(
                SqliteConnectOptions::new()
                    .filename(database_location)
            )
            .await
            .unwrap(),
    );
    sqlx::migrate!().run(db.as_ref()).await?;
    log::debug!("Successfully opened database");

    return Ok(db);
}

fn touch(path: String) {
    std::fs::OpenOptions::new()
        .read(true)
        .create(true)
        .write(true)
        .open(path)
        .unwrap();
}
