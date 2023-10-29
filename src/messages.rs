use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{FileMeta};

use crate::database::queries;
use crate::types::*;
use crate::util::unix_to_humantime;

pub async fn receive_entity_id(
    db: Arc<DbConn>,
    bot: BotType,
    dialogue: DialogueWithState,
    msg: Message,
) -> HandlerResult {
    // Check if message is sticker or animation
    if msg.sticker().is_none() && msg.animation().is_none() {
        bot.send_message(msg.chat.id, "Please send me a sticker or animation")
            .await?;
        return Ok(());
    }

    let sticker = msg.sticker().unwrap().file.clone();
    let user_id = msg.from().unwrap().id.to_string();

    log::debug!(
        "Got entity: {:?} file: {:?} from: {:?}",
        sticker.unique_id,
        sticker.id,
        user_id
    );

    let mut current_tags =
        queries::get_tags(&db, user_id.clone(), sticker.unique_id.clone()).await?;
    current_tags.sort();

    log::debug!("Current tags: {:?}", current_tags);

    let entity_usage =
        queries::get_entity_usage(&db, user_id.clone(), sticker.unique_id.clone()).await?;

    if current_tags.len() > 0 {
        bot.send_message(
            msg.chat.id,
            format!(
                "Your current tags for this sticker are: <b>{}</b>\n\
                You've used this sticker <code>{}</code> times\n\
                You've last used this sticker <code>{}</code>",
                current_tags.join(", "),
                entity_usage.count,
                unix_to_humantime(entity_usage.last_used)
            ),
        )
        .await?;
    }

    bot.send_message(
        msg.chat.id,
        "Which tags do you want to add to this sticker?\n\
        - Make the first tag <code>replace</code>, to replace all tags\n\
        - Make the first tag <code>clear</code>, to remove all existing tags\n\
        - Start the tag with <code>-</code> to remove an existing tag",
    )
    .await?;

    dialogue
        .update(ConversationState::ReceiveEntityTags { entity: sticker })
        .await?;

    Ok(())
}

pub async fn receive_entity_tags(
    db: Arc<DbConn>,
    bot: BotType,
    dialogue: DialogueWithState,
    msg: Message,
    entity: FileMeta,
) -> HandlerResult {
    if msg.text().is_none() {
        bot.send_message(
            msg.chat.id,
            "Please send me a space seperated list of tags or send <code>cancel</code> to cancel",
        )
        .await?;
        return Ok(());
    }

    let user_id = msg.from().unwrap().id.to_string();
    let mut tags: Vec<String> = msg
        .text()
        .unwrap()
        .to_lowercase()
        .replace(",", " ")
        .split(" ")
        .map(|s| s.trim().to_string())
        .filter(|s| s.len() > 0)
        .collect();

    log::debug!("Got tags: {:?} from {:?}", tags, user_id);

    if tags[0] == "cancel" {
        bot.send_message(msg.chat.id, "Cancelled").await?;
        dialogue.update(ConversationState::ReceiveEntityID).await?;
        return Ok(());
    }

    if tags[0] == "replace" || tags[0] == "clear" {
        log::debug!("Wiping tags");
        queries::wipe_tags(&db, user_id.clone(), entity.unique_id.clone()).await?;

        if tags[0] == "clear" {
            bot.send_message(msg.chat.id, "Cleared all tags for this sticker")
                .await?;
            dialogue.update(ConversationState::ReceiveEntityID).await?;
            return Ok(());
        }

        tags.remove(0);
    }

    if tags.len() == 0 {
        bot.send_message(msg.chat.id, "No tags provided").await?;
        return Ok(());
    }

    // split the tags into add and remove
    let remove_tags = tags
        .iter()
        .filter(|tag| tag.starts_with("-"))
        .map(|tag| tag.replace("-", ""))
        .collect::<Vec<String>>();

    let add_tags = tags
        .iter()
        .filter(|tag| !tag.starts_with("-"))
        .map(|tag| tag.to_string())
        .collect::<Vec<String>>();

    log::debug!("Removing tags: {:?}", remove_tags);
    log::debug!("Adding tags: {:?}", add_tags);

    queries::insert_tags(
        &db,
        user_id.clone(),
        entity.unique_id.clone(),
        entity.id.clone(),
        add_tags,
    )
    .await?;

    queries::remove_tags(&db, user_id.clone(), entity.unique_id.clone(), remove_tags).await?;

    tags = queries::get_tags(&db, user_id.clone(), entity.unique_id.clone()).await?;
    tags.sort();

    log::debug!("New tags for user {:?} is {:?}", user_id, tags);

    bot.send_message(
        msg.chat.id,
        format!(
            "The new tags for this sticker are now: <b>{}</b>",
            tags.join(", ")
        ),
    )
    .await?;
    dialogue.update(ConversationState::ReceiveEntityID).await?;
    Ok(())
}
