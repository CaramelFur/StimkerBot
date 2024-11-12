use std::sync::Arc;

use anyhow::{bail, Result};
use teloxide::{
  net::Download,
  requests::Requester,
  types::{InputFile, Message},
};

use crate::{
  database::{import, queries},
  types::{BotType, ConversationState, DbConn, DialogueWithState},
  util::get_unix,
};

use super::send_message::BetterSendMessage;

pub async fn receive_qs_import(
  db: Arc<DbConn>,
  bot: BotType,
  dialogue: DialogueWithState,
  msg: Message,
) -> Result<()> {
  dialogue.update(ConversationState::ReceiveEntityId).await?;

  let file_data = extract_file(&bot, &msg).await?;

  let user_id = msg.from.as_ref().unwrap().id.to_string();

  bot
    .send_message_easy(msg.chat.id, format!("Importing your entities..."))
    .await?;

  let result = import::quickstickbot_import(&db, user_id, file_data).await;
  match result {
    Ok(_) => {
      bot
        .send_message_easy(msg.chat.id, format!("Imported your entities!"))
        .await?;
    }
    Err(e) => {
      bot
        .send_message_easy(msg.chat.id, format!("Failed to import your entities"))
        .await?;
      log::error!("Failed to import entities: {:?}", e);
    }
  };

  Ok(())
}

pub async fn receive_bot_import(
  db: Arc<DbConn>,
  bot: BotType,
  dialogue: DialogueWithState,
  msg: Message,
) -> Result<()> {
  dialogue.update(ConversationState::ReceiveEntityId).await?;

  let file_data = extract_file(&bot, &msg).await?;

  let user_id = msg.from.as_ref().unwrap().id.to_string();

  bot
    .send_message_easy(msg.chat.id, format!("Importing your entities..."))
    .await?;

  let result = import::import(&db, user_id, file_data).await;
  match result {
    Ok(_) => {
      bot
        .send_message_easy(msg.chat.id, format!("Imported your entities!"))
        .await?;
    }
    Err(e) => {
      bot
        .send_message_easy(msg.chat.id, format!("Failed to import your entities"))
        .await?;
      log::error!("Failed to import entities: {:?}", e);
    }
  };

  Ok(())
}

pub async fn send_bot_export(db: &DbConn, bot: &BotType, msg: &Message) -> Result<()> {
  bot
    .send_message_easy(msg.chat.id, "Exporting your entities...")
    .await?;

  let user_id = msg.from.as_ref().unwrap().id.to_string();

  let data = import::export(&db, user_id).await?;

  bot
    .send_document(
      msg.chat.id,
      InputFile::memory(data).file_name("export.stimkerbot"),
    )
    .await?;

  Ok(())
}

pub async fn send_fix_entities(db: &DbConn, bot: &BotType, msg: &Message) -> Result<()> {
  let user_id = msg.from.as_ref().unwrap().id.to_string();

  let last_fixed_time = queries::get_last_fix_time(db, user_id.clone()).await?;

  if last_fixed_time > get_unix() - 600_000 {
    bot.send_message_easy(
          msg.chat.id,
          "You've already fixed your entities within the last 10 minutes, please wait a bit before trying again."
      )
      .await?;
    return Ok(());
  }

  bot
    .send_message_easy(msg.chat.id, "Fixing your entities...")
    .await?;

  let progress_message = bot.send_message(msg.chat.id, "Starting...").await?;

  let result = import::fix(db, &bot, &progress_message, user_id.to_owned()).await;

  if let Err(e) = result {
    bot
      .send_message_easy(msg.chat.id, format!("Failed to fix your entities"))
      .await?;
    log::error!("Failed to fix entities: {:?}", e);
    return Ok(());
  } else if let Ok(result) = result {
    queries::set_last_fix_time(db, user_id.to_owned(), get_unix()).await?;

    bot
      .send_message_easy(msg.chat.id, format!("Fixed {} entities!", result))
      .await?;
  }

  Ok(())
}

// ===========================================================

async fn extract_file(bot: &BotType, msg: &Message) -> Result<Vec<u8>> {
  // Check if message has a json attachment
  if msg.document().is_none() {
    bot
      .send_message_easy(msg.chat.id, "No file sent, operation cancelled")
      .await?;
    bail!("No file sent");
  }

  let doc = msg.document().unwrap();

  if doc.file.size > 50_000_000 {
    bot
      .send_message_easy(msg.chat.id, "File too large, operation cancelled")
      .await?;
    bail!("File too large");
  }

  let doc_data = bot.get_file(&doc.file.id).await?;
  let mut file_data = Vec::new();
  bot.download_file(&doc_data.path, &mut file_data).await?;

  Ok(file_data)
}
