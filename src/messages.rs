use sea_orm::DatabaseConnection;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::FileMeta;

use crate::database;
use crate::dialogue::*;

pub async fn receive_sticker_id(
    db: Arc<DatabaseConnection>,
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

    let sticker = msg.sticker().unwrap().file.clone();
    let user_id = msg.from().unwrap().id.to_string();

    log::debug!(
        "Got sticker: {:?} file: {:?} from: {:?}",
        sticker.unique_id,
        sticker.id,
        user_id
    );

    let mut current_tags = database::get_tags(&db, user_id, sticker.unique_id.clone()).await?;
    current_tags.sort();

    log::debug!("Current tags: {:?}", current_tags);

    if current_tags.len() > 0 {
        bot.send_message(
            msg.chat.id,
            format!(
                "Your current tags for this sticker are: {}",
                current_tags.join(", ")
            ),
        )
        .await?;
    }

    bot.send_message(
      msg.chat.id,
      "Which tags do you want to apply to this sticker?\n- Make the first tag `add`, to add to existing tags\n- Make the first tag `clear`, to remove all existing tags\n- Start the tag with `-` to remove an existing tag",
    )
    .await?;

    dialogue
        .update(ConversationState::ReceiveStickerTags { sticker })
        .await?;

    Ok(())
}

pub async fn receive_sticker_tags(
    db: Arc<DatabaseConnection>,
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    sticker: FileMeta,
) -> HandlerResult {
    if msg.text().is_none() {
        bot.send_message(msg.chat.id, "Please send me a space seperated list of tags")
            .await?;
        return Ok(());
    }

    let user_id = msg.from().unwrap().id.to_string();
    let mut tags: Vec<String> = msg
        .text()
        .unwrap()
        .replace(",", " ")
        .split(" ")
        .map(|s| s.trim().to_string())
        .collect();

    log::debug!("Got tags: {:?} from {:?}", tags, user_id);

    if tags[0] == "cancel" {
        bot.send_message(msg.chat.id, "Cancelled").await?;
        dialogue.update(ConversationState::ReceiveStickerID).await?;
        return Ok(());
    }

    if tags[0] == "add" {
        tags.remove(0);
    } else {
        log::debug!("Wiping tags");
        database::wipe_tags(&db, user_id.clone(), sticker.unique_id.clone()).await?;
    }

    if tags[0] == "clear" {
        bot.send_message(msg.chat.id, "Cleared all tags for this sticker")
            .await?;
        dialogue.update(ConversationState::ReceiveStickerID).await?;
        return Ok(());
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

    database::insert_tags(
        &db,
        user_id.clone(),
        sticker.unique_id.clone(),
        sticker.id.clone(),
        add_tags,
    )
    .await?;

    database::remove_tags(&db, user_id.clone(), sticker.unique_id.clone(), remove_tags).await?;

    tags = database::get_tags(&db, user_id.clone(), sticker.unique_id.clone()).await?;
    tags.sort();

    log::debug!("New tags for user {:?} is {:?}", user_id, tags);

    bot.send_message(
        msg.chat.id,
        format!("The new tags for this sticker are now: {}", tags.join(", ")),
    )
    .await?;
    dialogue.update(ConversationState::ReceiveStickerID).await?;
    Ok(())
}
