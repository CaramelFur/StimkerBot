use sea_orm::DatabaseConnection;
use std::sync::Arc;
use teloxide::prelude::*;

use teloxide::types::InlineQueryResult;
use teloxide::types::InlineQueryResultArticle;
use teloxide::types::InlineQueryResultCachedSticker;
use teloxide::types::InputMessageContent;
use teloxide::types::InputMessageContentText;

use crate::database;
use crate::dialogue::HandlerResult;

pub async fn handler_inline_query(
    db: Arc<DatabaseConnection>,
    bot: Bot,
    query: InlineQuery,
) -> HandlerResult {
    let user_id = query.from.id.to_string();

    // Check if query is empty
    if query.query.is_empty() || query.query.len() < 3 {
        if query.query == "*" {
            return handler_send_all(db, bot, query).await;
        }

        send_inline_results(
            &bot,
            query.id,
            vec![create_text_result(
                "Please enter atleast 3 characters".to_string(),
            )],
        )
        .await?;
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

    if stickers.len() == 0 {
        send_inline_results(
            &bot,
            query.id,
            vec![create_text_result("No stickers found".to_string())],
        )
        .await?;
        return Ok(());
    }

    log::info!("Found stickers: {:?}", stickers);

    let results = stickers.iter().map(|sticker| {
        InlineQueryResult::CachedSticker(InlineQueryResultCachedSticker {
            id: format!("{}", sticker.sticker_id.to_owned()),
            sticker_file_id: sticker.file_id.to_owned(),
            input_message_content: None,
            reply_markup: None,
        })
    });

    send_inline_results(&bot, query.id, results).await?;

    Ok(())
}

async fn handler_send_all(
    db: Arc<DatabaseConnection>,
    bot: Bot,
    query: InlineQuery,
) -> HandlerResult {
    let user_id = query.from.id.to_string();

    let stickers = database::list_stickers(&db, user_id).await?;

    if stickers.len() == 0 {
        send_inline_results(
            &bot,
            query.id,
            vec![create_text_result("No stickers found".to_string())],
        )
        .await?;
        return Ok(());
    }

    log::info!("Found stickers: {:?}", stickers);

    let results = stickers.iter().map(|sticker| {
        InlineQueryResult::CachedSticker(InlineQueryResultCachedSticker {
            id: format!("{}", sticker.sticker_id.to_owned()),
            sticker_file_id: sticker.file_id.to_owned(),
            input_message_content: None,
            reply_markup: None,
        })
    });

    send_inline_results(&bot, query.id, results).await
}

pub async fn handle_inline_choice(
    db: Arc<DatabaseConnection>,
    query: ChosenInlineResult,
) -> HandlerResult {
    log::info!("Chosen inline result: {:?} From: {:?}", query, query.from);

    database::increase_sticker_stat(&db, query.from.id.to_string(), query.result_id).await?;

    Ok(())
}

async fn send_inline_results<I, R>(bot: &Bot, inline_query_id: I, results: R) -> HandlerResult
where
    I: Into<String>,
    R: IntoIterator<Item = InlineQueryResult>,
{
    bot.answer_inline_query(inline_query_id, results)
        .cache_time(60)
        .is_personal(true)
        .await?;

    Ok(())
}

fn create_text_result(text: String) -> InlineQueryResult {
    InlineQueryResult::Article(InlineQueryResultArticle::new(
        "0",
        text,
        InputMessageContent::Text(InputMessageContentText::new("[error sending sticker]")),
    ))
}
