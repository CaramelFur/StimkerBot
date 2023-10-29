use dotenv::dotenv;
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::*;

mod database;
mod inline;
mod messages;
mod types;
mod util;

use types::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    pretty_env_logger::init();

    log::info!("Starting stimkerbot");

    let db = get_db().await?;

    let bot = Bot::from_env().parse_mode(teloxide::types::ParseMode::Html);

    let message_receive_sticker_id_tree = dptree::case![ConversationState::ReceiveEntityID]
        .endpoint({
            let db = db.clone(); // whyyyyy
            move |bot, dialogue, msg| messages::receive_entity_id(db.clone(), bot, dialogue, msg)
        });

    let message_receive_sticker_tags_tree =
        dptree::case![ConversationState::ReceiveEntityTags { entity, entity_type }].endpoint({
            let db = db.clone();
            move |bot, dialogue, (entity, entity_type), msg| {
                messages::receive_entity_tags(db.clone(), bot, dialogue, msg, entity, entity_type)
            }
        });

    let message_tree = Update::filter_message()
        .enter_dialogue::<Message, InMemStorage<ConversationState>, ConversationState>()
        .branch(message_receive_sticker_id_tree)
        .branch(message_receive_sticker_tags_tree);

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

    log::debug!("Starting dispatcher");

    Dispatcher::builder(bot, tree)
        .dependencies(dptree::deps![InMemStorage::<ConversationState>::new()])
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
            .connect(&format!("sqlite://{}", database_location))
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
