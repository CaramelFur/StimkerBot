use dotenv::dotenv;
use sea_orm::Database;
use std::sync::Arc;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::*;

mod database;
mod dialogue;
mod inline;
mod messages;

use dialogue::*;

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    log::info!("Starting stimkerbot");

    let db = Arc::new(Database::connect("sqlite://bitch.db").await.unwrap());
    let bot = Bot::from_env();

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

    Dispatcher::builder(bot, tree)
        .dependencies(dptree::deps![InMemStorage::<ConversationState>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
