use teloxide::{types::Message, requests::Requester, ApiError, RequestError};

use crate::{types::{DbConn, BotType, HandlerResult, InlineSearchQuery, EntitySort}, database::queries::find_entities};

pub async fn fix(
  db: &DbConn,
  bot: &BotType,
  progress_message: &Message,
  user_id: String,
) -> HandlerResult<i32> {
  let mut i: i32 = 0;
  let mut file_ids_to_remove: Vec<String> = Vec::new();
  loop {
      let entities = find_entities(
          db,
          user_id.to_owned(),
          InlineSearchQuery {
              sort: EntitySort::LastAdded,
              get_all: true,
              ..Default::default()
          },
          i,
      )
      .await?;
      if entities.len() == 0 {
          break;
      }

      bot.edit_message_text(
          progress_message.chat.id,
          progress_message.id,
          format!(
              "Checking files {}-{}",
              i * 50 + 1,
              i * 50 + entities.len() as i32
          ),
      )
      .await?;

      i += 1;

      for entity in entities {
          log::trace!("Checking file {:?}", entity.file_id);
          let file = bot.get_file(&entity.file_id).await;
          if let Err(RequestError::Api(ApiError::Unknown(e))) = &file {
              if e.contains("wrong file_id") {
                  file_ids_to_remove.push(entity.file_id);
                  continue;
              }
          }
          file?;
      }
  }

  let fixed = file_ids_to_remove.len() as i32;

  for file_id in file_ids_to_remove {
      log::info!("Removing file {:?}", file_id);
      remove_file_id(db, file_id).await?;
  }

  Ok(fixed)
}

async fn remove_file_id(db: &DbConn, file_id: String) -> HandlerResult {
  // Remove the enitity_data with the file_id, and all tables referencing its combo_id

  let mut transaction = db.begin().await?;

  // Delete all entity_main where combo_id is in entity_data with file_id
  sqlx::query(
      "DELETE FROM entity_main WHERE combo_id IN \
      (SELECT combo_id FROM entity_data JOIN entity_file ON entity_file.entity_id = entity_data.entity_id \
       WHERE entity_file.file_id = $1)")
      .bind(&file_id)
      .execute(transaction.as_mut())
      .await?;

  // Delete all entity_data where entity_id is in entity_file with file_id
  sqlx::query(
      "DELETE FROM entity_data WHERE entity_id IN \
      (SELECT entity_id FROM entity_file WHERE file_id = $1)",
  )
  .bind(&file_id)
  .execute(transaction.as_mut())
  .await?;

  // Delete all entity_file with file_id
  sqlx::query("DELETE FROM entity_file WHERE file_id = $1")
      .bind(&file_id)
      .execute(transaction.as_mut())
      .await?;

  transaction.commit().await?;

  Ok(())
}
