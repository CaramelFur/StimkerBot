use dotenv::dotenv;
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use std::sync::Arc;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::*;

mod database;
mod dialogue;
mod inline;
mod messages;
mod util;

use dialogue::*;

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    log::info!("Starting stimkerbot");

    let database_location =
        std::env::var("DATABASE_LOCATION").expect("DATABASE_LOCATION must be set");
    log::debug!("Database location: {:?}", database_location);

    log::debug!("Opening/creating and migrating database");
    std::fs::OpenOptions::new().read(true).create(true).write(true).open(&database_location).unwrap();
    let db = Arc::new(
        Database::connect(format!("sqlite://{}", database_location))
            .await
            .unwrap(),
    );
    Migrator::up(db.as_ref(), None).await.unwrap();
    log::debug!("Successfully opened database");

    let bot = Bot::from_env()
        .parse_mode(teloxide::types::ParseMode::Html);

    let message_receive_sticker_id_tree = dptree::case![ConversationState::ReceiveStickerID]
        .endpoint({
            let db = db.clone(); // whyyyyy
            move |bot, dialogue, msg| messages::receive_sticker_id(db.clone(), bot, dialogue, msg)
        });

    let message_receive_sticker_tags_tree =
        dptree::case![ConversationState::ReceiveStickerTags { sticker }].endpoint({
            let db = db.clone();
            move |bot, dialogue, sticker, msg| {
                messages::receive_sticker_tags(db.clone(), bot, dialogue, msg, sticker)
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
}
