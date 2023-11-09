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

    // Parse query.offset as i32 or fallback to 0
    let page = query.offset.parse::<i32>().unwrap_or(0);

    // Check if query is empty
    if query.query.len() < 3 {
        log::debug!("Query too short: \"{:?}\" for {:?}", query.query, user_id);

        send_text_result(&bot, query.id, "Please enter atleast 3 characters").await?;
        return Ok(());
    }

    // Split query by spaces into string vector
    let search_query = parse_search(&query.query);

    log::debug!(
        "Got inline query: {:?} from {:?} at page {}",
        query,
        user_id,
        page
    );

    let entities = queries::find_entities(&db, user_id, search_query.to_owned(), page).await?;

    if entities.len() == 0 && page == 0 {
        send_text_result(&bot, query.id, "No stickers found").await?;
        return Ok(());
    }

    log::debug!("Found stickers: {:?}", entities);

    let results = entities.iter().map(|sticker| sticker.to_inline());

    send_inline_results(
        &bot,
        query.id,
        results,
        if search_query.sort != EntitySort::Random {
            Some(page + 1)
        } else {
            None
        },
    )
    .await?;

    Ok(())
}

fn parse_search(input: &String) -> InlineSearchQuery {
    let mut query = InlineSearchQuery::default();

    let tags_all: Vec<String> = input
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
            "random" | "rnd" => {
                query.sort = EntitySort::Random;
                false
            }
            _ => true,
        })
        .collect();

    let mut tags_positive = Vec::new();
    let mut tags_negative = Vec::new();

    for tag in tags_all {
        if tag.starts_with("-") {
            tags_negative.push(tag[1..].to_owned());
        } else {
            tags_positive.push(tag);
        }
    }

    query.tags = tags_positive;
    query.negative_tags = tags_negative;

    query
}

// ===================================================================

pub async fn handle_inline_choice(db: Arc<DbConn>, query: ChosenInlineResult) -> HandlerResult {
    let user_id = query.from.id.to_string();

    log::debug!("Chosen inline result: {:?} by user {:?}", query, user_id);

    queries::increase_entity_stat(&db, user_id, query.result_id).await?;

    Ok(())
}

async fn send_inline_results<I, R>(
    bot: &BotType,
    inline_query_id: I,
    results: R,
    next_page: Option<i32>,
) -> HandlerResult
where
    I: Into<String>,
    R: IntoIterator<Item = InlineQueryResult>,
{
    let mut payload = payloads::AnswerInlineQuery::new(inline_query_id, results)
        .cache_time(5)
        .is_personal(true);

    if let Some(next_page) = next_page {
        payload = payload.next_offset(next_page.to_string());
    }

    <Bot as Requester>::AnswerInlineQuery::new(bot.inner().clone(), payload).await?;

    Ok(())
}

async fn send_text_result<I, R>(bot: &BotType, inline_query_id: I, text: R) -> HandlerResult
where
    I: Into<String>,
    R: Into<String>,
{
    let results = vec![InlineQueryResult::Article(InlineQueryResultArticle::new(
        "0",
        text,
        InputMessageContent::Text(InputMessageContentText::new("[error sending sticker]")),
    ))];

    <Bot as Requester>::AnswerInlineQuery::new(
        bot.inner().clone(),
        payloads::AnswerInlineQuery::new(inline_query_id, results)
            .cache_time(5)
            .is_personal(true)
            .switch_pm_text("Add a new sticker")
            .switch_pm_parameter("bot"),
    )
    .await?;

    Ok(())
}
