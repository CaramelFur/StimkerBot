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
      "Which tags do you want to apply to this sticker?\n- Make the first tag `add`, to add to existing tags\n- Make the first tag `clear`, to remove all existing tags",
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
        .split(" ")
        .map(|s| s.trim().to_string())
        .collect();

    log::debug!("Got tags: {:?} from {:?}", tags, user_id);

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

    database::insert_tags(
        &db,
        user_id.clone(),
        sticker.unique_id.clone(),
        sticker.id.clone(),
        tags.clone(),
    )
    .await?;

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
