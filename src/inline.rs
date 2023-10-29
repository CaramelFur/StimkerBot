use std::sync::Arc;
use teloxide::payloads;
use teloxide::prelude::*;

use teloxide::types::*;

use crate::database::queries;
use crate::types::BotType;
use crate::types::DbConn;
use crate::types::EntitySort;
use crate::types::HandlerResult;

pub async fn handler_inline_query(
    db: Arc<DbConn>,
    bot: BotType,
    query: InlineQuery,
) -> HandlerResult {
    let user_id = query.from.id.to_string();

    // Check if query is empty
    if query.query.len() < 3 {
        if query.query == "*" {
            return handler_send_all(db, bot, query).await;
        }

        log::debug!("Query too short: \"{:?}\" for {:?}", query.query, user_id);

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
        .to_lowercase()
        .replace(",", " ")
        .split(" ")
        .map(|s| s.trim().to_string())
        .collect();

    log::debug!("Got inline query: {:?} from {:?}", query, user_id);

    let entities =
        queries::find_entities(&db, user_id, tags, 0, EntitySort::MostUsed).await?;

    if entities.len() == 0 {
        send_inline_results(
            &bot,
            query.id,
            vec![create_text_result("No stickers found".to_string())],
        )
        .await?;
        return Ok(());
    }

    log::debug!("Found stickers: {:?}", entities);

    let results = entities.iter().map(|sticker| sticker.to_inline());

    send_inline_results(&bot, query.id, results).await?;

    Ok(())
}

async fn handler_send_all(db: Arc<DbConn>, bot: BotType, query: InlineQuery) -> HandlerResult {
    let user_id = query.from.id.to_string();

    log::debug!("Sending all stickers for {:?}", user_id);

    let entities =
        queries::list_entities(&db, user_id, 0, EntitySort::MostUsed).await?;

    if entities.len() == 0 {
        send_inline_results(
            &bot,
            query.id,
            vec![create_text_result("No stickers found".to_string())],
        )
        .await?;
        return Ok(());
    }

    log::debug!("Found all stickers: {:?}", entities);

    let results = entities.iter().map(|sticker| sticker.to_inline());

    send_inline_results(&bot, query.id, results).await
}

pub async fn handle_inline_choice(db: Arc<DbConn>, query: ChosenInlineResult) -> HandlerResult {
    let user_id = query.from.id.to_string();

    log::debug!("Chosen inline result: {:?} by user {:?}", query, user_id);

    queries::increase_entity_stat(&db, user_id, query.result_id).await?;

    Ok(())
}

async fn send_inline_results<I, R>(bot: &BotType, inline_query_id: I, results: R) -> HandlerResult
where
    I: Into<String>,
    R: IntoIterator<Item = InlineQueryResult>,
{
    <Bot as Requester>::AnswerInlineQuery::new(
        bot.inner().clone(),
        payloads::AnswerInlineQuery::new(inline_query_id, results)
            .cache_time(5)
            .is_personal(true),
    )
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
