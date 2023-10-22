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

    let mut current_tags = database::get_tags(
        &db,
        msg.from().unwrap().id.to_string(),
        sticker.unique_id.clone(),
    )
    .await?;
    current_tags.sort();

    if current_tags.len() > 0 {
        println!("Sending message");

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
    let user_id = msg.from().unwrap().id.to_string();

    // Check if message is text
    if msg.text().is_none() {
        bot.send_message(msg.chat.id, "Please send me a space seperated list of tags")
            .await?;
        return Ok(());
    }

    // Split text by spaces into string vector
    let mut tags: Vec<String> = msg
        .text()
        .unwrap()
        .split(" ")
        .map(|s| s.trim().to_string())
        .collect();

    // If first tag is 'add', don't clear existing tags, and remove 'add' from tags
    if tags[0] == "add" {
        tags.remove(0);
    } else {
        // If first tag is not 'add', clear existing tags
        database::wipe_tags(&db, user_id.clone(), sticker.unique_id.clone()).await?;
    }

    if tags[0] == "clear" {
        bot.send_message(msg.chat.id, "Cleared all tags for this sticker")
            .await?;
        dialogue.update(ConversationState::ReceiveStickerID).await?;
        return Ok(());
    }

    // Insert new sticker tags
    database::insert_tags(
        &db,
        user_id.clone(),
        sticker.unique_id.clone(),
        sticker.id.clone(),
        tags.clone(),
    )
    .await?;

    // Get all tags for this sticker
    tags = database::get_tags(&db, user_id.clone(), sticker.unique_id.clone()).await?;
    tags.sort();

    // Reply by joining the strings by commas
    bot.send_message(
        msg.chat.id,
        format!("The new tags for this sticker are now: {}", tags.join(", ")),
    )
    .await?;
    dialogue.update(ConversationState::ReceiveStickerID).await?;
    Ok(())
}
