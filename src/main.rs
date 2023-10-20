use dotenv::dotenv;
use sea_orm::Database;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::*;
use teloxide::types::InlineQueryResult;
use teloxide::types::InlineQueryResultCachedSticker;

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
    dotenv().ok();
    pretty_env_logger::init();

    log::info!("Starting stimkerbot");

    let db = Arc::new(Database::connect("sqlite://bitch.db").await.unwrap());

    let bot = Bot::from_env();

    let message_handler = Update::filter_message()
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
        );

    let inline_handler = Update::filter_inline_query().endpoint({
        let db = db.clone();
        move |bot, query| handler_inline_query(db.clone(), bot, query)
    });

    let handler = dptree::entry()
        .branch(message_handler)
        .branch(inline_handler);

    Dispatcher::builder(bot, handler)
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

async fn handler_inline_query(
    db: Arc<DatabaseConnection>,
    bot: Bot,
    query: InlineQuery,
) -> HandlerResult {
    let user_id = query.from.id.to_string();

    // Check if query is empty
    if query.query.is_empty() {
        bot.answer_inline_query(query.id, vec![]).await?;
        return Ok(());
    }

    // Split query by spaces into string vector
    let tags: Vec<String> = query
        .query
        .split(" ")
        .map(|s| s.trim().to_string())
        .collect();

    log::info!("Got inline query: {:?}", tags);

    let stickers = database::find_stickers(&db, user_id, tags).await?;

    log::info!("Found stickers: {:?}", stickers);

    let mut i = 0;
    let results = stickers.iter().map(|sticker| {
        i += 1;
        InlineQueryResult::CachedSticker(InlineQueryResultCachedSticker {
            id: format!("{}", i),
            sticker_file_id: sticker.clone(),
            input_message_content: None,
            reply_markup: None,
        })
    });

    bot.answer_inline_query(query.id, results).await?;

    Ok(())
}
