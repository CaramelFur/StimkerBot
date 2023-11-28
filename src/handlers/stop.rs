use std::sync::Arc;

use teloxide::types::Message;

use crate::{types::{DbConn, BotType, DialogueWithState, HandlerResult, ConversationState}, handlers::send_message::BetterSendMessage as _, database::queries};

pub async fn verify_stop(
  db: Arc<DbConn>,
  bot: BotType,
  dialogue: DialogueWithState,
  msg: Message,
) -> HandlerResult {
  dialogue.update(ConversationState::ReceiveEntityId).await?;

  if msg.text().is_none() || msg.text().unwrap() != "I WANT TO DELETE EVERYTHING" {
      bot.send_message_easy(msg.chat.id, "Stop action cancelled")
          .await?;
      return Ok(());
  }

  let user_id = msg.from().unwrap().id.to_string();

  log::debug!("Wiping user {:?}", user_id);

  queries::wipe_user(&db, user_id.clone()).await?;

  bot.send_message_easy(msg.chat.id, "All your data has been wiped")
      .await?;

  Ok(())
}