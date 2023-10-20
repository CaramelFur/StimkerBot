use sea_orm::Database;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::net;
use teloxide::prelude::*;

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

    let db = Arc::new(Database::connect("sqlite://bitch.db").await.unwrap());

    let bot = Bot::with_client(
        "6747586175:AAHv2mtzDQobtCHG7qpkspL4GbNQEfThIVc",
        net::client_from_env(),
    );

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<ConversationState>, ConversationState>()
            .branch(
                dptree::case![ConversationState::ReceiveStickerID].endpoint({
                    let db = db.clone(); // whyyyyy
                    move |bot, dialogue, msg| receive_sticker_id(db.clone(), bot, dialogue, msg)
                }),
            )
            .branch(
                dptree::case![ConversationState::ReceiveStickerTags { sticker_id }].endpoint({
                    let db = db.clone();
                    move |bot, dialogue, sticker_id, msg| {
                        receive_sticker_tags(db.clone(), bot, dialogue, msg, sticker_id)
                    }
                }),
            ),
    )
    .dependencies(dptree::deps![InMemStorage::<ConversationState>::new()])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

async fn receive_sticker_id(
    _: Arc<DatabaseConnection>,
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
) -> HandlerResult {
    // Check if message is sticker
    if msg.sticker().is_none() {
        bot.send_message(msg.chat.id, "Please send me a sticker")
            .await?;
        return Ok(());
    }

    let sticker_id = msg.sticker().unwrap().file.id.clone();

    bot.send_message(
        msg.chat.id,
        "Alright, which tags would you like to associate with this sticker?",
    )
    .await?;
    dialogue
        .update(ConversationState::ReceiveStickerTags { sticker_id })
        .await?;
    Ok(())
}

async fn receive_sticker_tags(
    db: Arc<DatabaseConnection>,
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    sticker_id: String,
) -> HandlerResult {
    // Check if message is text
    if msg.text().is_none() {
        bot.send_message(msg.chat.id, "Please send me a space seperated list of tags")
            .await?;
        return Ok(());
    }

    // Split text by spaces into string vector
    let tags: Vec<String> = msg
        .text()
        .unwrap()
        .split(" ")
        .map(|s| s.trim().to_string())
        .collect();

    // Clear exisiting sticker tags
    database::wipe_tags(&db, msg.from().unwrap().id.to_string(), sticker_id.clone()).await?;

    // Insert new sticker tags
    database::insert_tags(
        &db,
        msg.from().unwrap().id.to_string(),
        sticker_id.clone(),
        tags.clone(),
    )
    .await?;

    // Reply by joining the strings by commas
    bot.send_message(
        msg.chat.id,
        format!("The new tags for this sticker are now: {}", tags.join(", ")),
    )
    .await?;
    dialogue.update(ConversationState::ReceiveStickerID).await?;
    Ok(())
}
