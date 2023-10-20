use sea_orm::Database;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::net;
use teloxide::prelude::*;

use entity::sticker_tag;
use sea_orm::entity::prelude::*;

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
type MyDialogue = Dialogue<ConversationState, InMemStorage<ConversationState>>;

mod database;

#[derive(Clone, Default)]
pub enum ConversationState {
    #[default]
    ReceiveStickerID,
    ReceiveStickerTags {
        sticker_id: String,
    },
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting throw dice bot...");

    let mut db = Database::connect("sqlite://bitch.db").await.unwrap();

    database::insert_tag(&db, "one".into(), "two".into(), "three".into()).await;

    let bot = Bot::with_client(
        "6747586175:AAHv2mtzDQobtCHG7qpkspL4GbNQEfThIVc",
        net::client_from_env(),
    );

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<ConversationState>, ConversationState>()
            .branch(dptree::case![ConversationState::ReceiveStickerID].endpoint(receive_sticker_id))
            .branch(
                dptree::case![ConversationState::ReceiveStickerTags { sticker_id }]
                    .endpoint(receive_sticker_tags),
            ),
    )
    .dependencies(dptree::deps![InMemStorage::<ConversationState>::new()])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

async fn receive_sticker_id(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    // Check if message is sticker
    if msg.sticker().is_none() {
        bot.send_message(msg.chat.id, "Please send me a sticker")
            .await?;
        return Ok(());
    }

    let sticker_id = msg.sticker().unwrap().file.id.clone();

    bot.send_message(msg.chat.id, "Received Sticker").await?;
    dialogue
        .update(ConversationState::ReceiveStickerTags { sticker_id })
        .await?;
    Ok(())
}

async fn receive_sticker_tags(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    // Check if message is text
    if msg.text().is_none() {
        bot.send_message(msg.chat.id, "Please send me some text")
            .await?;
        return Ok(());
    }

    // Split text by spaces into string vector
    let tags: Vec<&str> = msg.text().unwrap().split(" ").collect();

    // Reply by joining the strings by commas
    bot.send_message(msg.chat.id, tags.join(", ")).await?;
    dialogue.update(ConversationState::ReceiveStickerID).await?;
    Ok(())
}
