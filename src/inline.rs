use std::sync::Arc;
use teloxide::payloads;
use teloxide::prelude::*;

use teloxide::types::*;

use crate::database::entities::EntityType;
use crate::database::queries;
use crate::types::BotType;
use crate::types::DbConn;
use crate::types::EntitySort;
use crate::types::HandlerResult;
use crate::types::InlineSearchQuery;

pub async fn handler_inline_query(
    db: Arc<DbConn>,
    bot: BotType,
    query: InlineQuery,
) -> HandlerResult {
    let user_id = query.from.id.to_string();

    // Check if query is empty
    if query.query.len() < 3 {
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
    let search_query = parse_search(&query.query);

    log::debug!("Got inline query: {:?} from {:?}", query, user_id);

    let entities = queries::find_entities(&db, user_id, search_query, 0).await?;

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

fn parse_search(input: &String) -> InlineSearchQuery {
    let mut query = InlineSearchQuery::default();

    let tags: Vec<String> = input
        .to_lowercase()
        .replace(",", " ")
        .split(" ")
        .map(|s| s.trim().to_string().to_owned())
        .filter(|tag| !tag.is_empty())
        .filter(|tag| match tag.as_str() {
            "all" => {
                query.get_all = true;
                false
            }
            "sticker" | "stk" => {
                query.entity_type = Some(EntityType::Sticker);
                false
            }
            "animation" | "gif" => {
                query.entity_type = Some(EntityType::Animation);
                false
            }
            "photo" | "pic" => {
                query.entity_type = Some(EntityType::Photo);
                false
            }
            "video" | "vid" => {
                query.entity_type = Some(EntityType::Video);
                false
            }
            "most_used" | "mu" => {
                query.sort = EntitySort::MostUsed;
                false
            }
            "least_used" | "lu" => {
                query.sort = EntitySort::LeastUsed;
                false
            }
            "last_added" | "la" => {
                query.sort = EntitySort::LastAdded;
                false
            }
            "first_added" | "fa" => {
                query.sort = EntitySort::FirstAdded;
                false
            }
            "last_used" | "nu" => {
                query.sort = EntitySort::LastUsed;
                false
            }
            "first_used" | "ou" => {
                query.sort = EntitySort::FirstUsed;
                false
            }
            _ => true,
        })
        .collect();

    query.tags = tags;

    query
}

// ===================================================================

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
