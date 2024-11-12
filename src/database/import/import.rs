use crate::{
  database::entities::{Entity, EntityType},
  types::DbConn,
};
use anyhow::Result;
use flate2::read::GzDecoder;
use sqlx::QueryBuilder;
use std::collections::HashSet;
use std::io::Read;

use super::types::{BotImport, ImportItem, QSBotImport};

pub async fn quickstickbot_import(db: &DbConn, user_id: String, file: Vec<u8>) -> Result<()> {
  // Parse the file
  let qs_import: QSBotImport = serde_json::from_slice(&file)?;

  // Insert the file into the database
  let mut bot_import: BotImport = Vec::new();

  for item in qs_import {
    let entity_type = Entity::file_id_to_type(&item.file_id)?;

    bot_import.push(ImportItem {
      entity_id: item.id,
      file_id: item.file_id,
      entity_type: entity_type,
      tags: item.tags,
      count: 0,
      last_used: 0,
      created_at: 0,
    });
  }

  import_botimport(db, user_id, bot_import).await?;

  Ok(())
}

pub async fn import(db: &DbConn, user_id: String, file: Vec<u8>) -> Result<()> {
  let mut decompressor = GzDecoder::new(file.as_slice());
  let mut decompressed = Vec::new();
  decompressor.read_to_end(&mut decompressed)?;

  let import: BotImport = serde_json::from_slice(&decompressed)?;

  import_botimport(db, user_id, import).await
}

async fn import_botimport(db: &DbConn, user_id: String, import: BotImport) -> Result<()> {
  log::debug!("Importing {} items for user {}", import.len(), user_id);

  let mut transaction = db.begin().await?;

  log::debug!("[IMPORT] Inserting tags");

  // Collect all unique tags, and split them into sets of 1000 tags
  // Then insert
  {
    let unqiue_tags: Vec<String> = {
      let unqiue_tags: HashSet<String> = import
        .iter()
        .flat_map(|item| item.tags.to_owned())
        .map(|tag| {
          tag
            .to_lowercase()
            .replace(",", "")
            .replace(" ", "")
            .trim()
            .to_string()
        })
        .collect();
      unqiue_tags.into_iter().collect()
    };
    for chunk in unqiue_tags.chunks(1000) {
      let mut insert_tag_query = QueryBuilder::new("INSERT OR IGNORE INTO entity_tag (tag_name) ");
      insert_tag_query.push_values(chunk, |mut b, tag_name| {
        b.push_bind(tag_name);
      });
      insert_tag_query
        .build()
        .execute(transaction.as_mut())
        .await?;
    }
  }

  log::debug!("[IMPORT] Inserting entities");

  // Insert all files
  {
    struct ImportEntity {
      entity_id: String,
      file_id: String,
      entity_type: EntityType,
    }

    let files: Vec<ImportEntity> = import
      .iter()
      .map(|item| ImportEntity {
        entity_id: item.entity_id.clone(),
        file_id: item.file_id.clone(),
        entity_type: item.entity_type.clone(),
      })
      .collect();
    for chunk in files.chunks(1000) {
      let mut insert_file_query =
        QueryBuilder::new("INSERT OR IGNORE INTO entity_file (entity_id, file_id, entity_type) ");
      insert_file_query.push_values(chunk, |mut b, file| {
        b.push_bind(&file.entity_id);
        b.push_bind(&file.file_id);
        b.push_bind(&file.entity_type);
      });
      insert_file_query
        .build()
        .execute(transaction.as_mut())
        .await?;
    }
  }

  log::debug!("[IMPORT] Inserting combos");

  // Insert all user entity combos
  {
    struct ImportEntity {
      entity_id: String,
      count: i64,
      last_used: i64,
      created_at: i64,
    }

    let combos: Vec<ImportEntity> = import
      .iter()
      .map(|item| ImportEntity {
        entity_id: item.entity_id.clone(),
        count: item.count,
        last_used: item.last_used,
        created_at: item.created_at,
      })
      .collect();
    for chunk in combos.chunks(1000) {
      let mut insert_combo_query = QueryBuilder::new(
        "INSERT OR IGNORE INTO entity_data (user_id, entity_id, count, last_used, created_at) ",
      );
      insert_combo_query.push_values(chunk, |mut b, combo| {
        b.push_bind(&user_id);
        b.push_bind(&combo.entity_id);
        b.push_bind(&combo.count);
        b.push_bind(&combo.last_used);
        b.push_bind(&combo.created_at);
      });
      insert_combo_query
        .build()
        .execute(transaction.as_mut())
        .await?;
    }
  }

  log::debug!("[IMPORT] Inserting main");

  // Insert all tag relations
  {
    for entity in import {
      let mut insert_main_query = QueryBuilder::new(
        "INSERT OR IGNORE INTO entity_main (combo_id, tag_id) \
                    SELECT (SELECT combo_id FROM entity_data WHERE entity_id = ",
      );
      insert_main_query.push_bind(&entity.entity_id);
      insert_main_query.push(" AND user_id = ");
      insert_main_query.push_bind(&user_id);
      insert_main_query.push("), tag_id FROM (SELECT tag_id FROM entity_tag WHERE tag_name IN (");
      let mut seperator = insert_main_query.separated(", ");
      entity.tags.iter().for_each(|tag_name| {
        seperator.push_bind(tag_name);
      });
      insert_main_query.push("))");

      insert_main_query
        .build()
        .execute(transaction.as_mut())
        .await?;
    }
  }

  log::debug!("[IMPORT] Committing");

  transaction.commit().await?;

  log::debug!("[IMPORT] Done");

  Ok(())
}
