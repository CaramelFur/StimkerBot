use serde;
use serde::{Deserialize, Serialize};
use sqlx::QueryBuilder;

use crate::types::{DbConn, HandlerResult};

use super::entities::EntityType;
use std::collections::HashSet;

// Serde object for [{"id":"AgADbgoAAtfwRQY","fileId":"CAACAgEAAxkBAAMYZT_z-AP_IWmM1aPKlzcIoEQsZ4UAAm4KAALX8EUG8J8EHnzA7KIwBA","tags":["tes"],"set":"XykYote","isAnimated":false}]
type QSBotImport = Vec<QSBotImportItem>;

#[derive(Deserialize, Debug)]
struct QSBotImportItem {
    id: String,
    #[serde(rename = "fileId")]
    file_id: String,
    tags: Vec<String>,
    #[serde(rename = "set")]
    _set: String,
    #[serde(rename = "isAnimated")]
    _is_animated: bool,
}

pub async fn import_qsbot(db: &DbConn, user_id: String, file: Vec<u8>) -> HandlerResult {
    // Parse the file
    let qs_import: QSBotImport = serde_json::from_slice(&file)?;

    // Insert the file into the database
    let bot_import: BotImport = qs_import
        .into_iter()
        .map(|item| ImportItem {
            entity_id: item.id,
            file_id: item.file_id,
            entity_type: EntityType::Sticker,
            tags: item.tags,
            count: 0,
            last_used: 0,
            created_at: 0,
        })
        .collect();

    import_botimport(db, user_id, bot_import).await?;

    Ok(())
}

type BotImport = Vec<ImportItem>;

#[derive(Deserialize, Serialize, Debug)]
struct ImportItem {
    entity_id: String,

    file_id: String,
    entity_type: EntityType,

    tags: Vec<String>,

    count: i64,
    last_used: i64,
    created_at: i64,
}

pub async fn import_json(db: &DbConn, user_id: String, file: Vec<u8>) -> HandlerResult {
    // Parse the file
    let import: BotImport = serde_json::from_slice(&file)?;

    import_botimport(db, user_id, import).await
}

async fn import_botimport(db: &DbConn, user_id: String, import: BotImport) -> HandlerResult {
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
                .collect();
            unqiue_tags.into_iter().collect()
        };
        for chunk in unqiue_tags.chunks(1000) {
            let mut insert_tag_query =
                QueryBuilder::new("INSERT OR IGNORE INTO entity_tag (tag_name) ");
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
            let mut insert_file_query = QueryBuilder::new(
                "INSERT OR IGNORE INTO entity_file (entity_id, file_id, entity_type) ",
            );
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
            let mut insert_main_query =
                QueryBuilder::new("INSERT OR IGNORE INTO entity_main (combo_id, tag_id) ");
            insert_main_query.push_values(entity.tags, |mut b, tag_name| {
                b.push("(SELECT combo_id FROM entity_data WHERE entity_id = ")
                    .push_bind_unseparated(entity.entity_id.clone())
                    .push_unseparated(" AND user_id = ")
                    .push_bind_unseparated(user_id.clone())
                    .push_unseparated(")");
                b.push("(SELECT tag_id FROM entity_tag WHERE tag_name = ")
                    .push_bind_unseparated(tag_name)
                    .push_unseparated(")");
            });
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

pub async fn export_botimport(db: &DbConn, user_id: String) -> HandlerResult<Vec<u8>> {
  // TODO:

    // "testing" to ascii u8
    let data = "testing";
    let vec: Vec<u8> = data.bytes().collect();
    Ok(vec)
}
