use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde;
use serde::{Deserialize, Serialize};
use sqlx::QueryBuilder;
use std::collections::HashSet;
use std::io::{Read, Write};

use crate::types::{DbConn, HandlerResult};

use super::entities::EntityType;

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
    #[serde(rename = "id")]
    entity_id: String,

    #[serde(rename = "fi")]
    file_id: String,
    #[serde(rename = "tp")]
    entity_type: EntityType,
    #[serde(rename = "t")]
    tags: Vec<String>,

    #[serde(rename = "c")]
    count: i64,
    #[serde(rename = "lu")]
    last_used: i64,
    #[serde(rename = "ca")]
    created_at: i64,
}

pub async fn import_json(db: &DbConn, user_id: String, file: Vec<u8>) -> HandlerResult {
    let mut decompressor = GzDecoder::new(file.as_slice());
    let mut decompressed = Vec::new();
    decompressor.read_to_end(&mut decompressed)?;

    let import: BotImport = serde_json::from_slice(&decompressed)?;

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
                .map(|tag| {
                    tag.to_lowercase()
                        .replace(",", "")
                        .replace(" ", "")
                        .trim()
                        .to_string()
                })
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
            let mut insert_main_query = QueryBuilder::new(
                "INSERT OR IGNORE INTO entity_main (combo_id, tag_id) \
                    SELECT (SELECT combo_id FROM entity_data WHERE entity_id = ",
            );
            insert_main_query.push_bind(&entity.entity_id);
            insert_main_query.push(" AND user_id = ");
            insert_main_query.push_bind(&user_id);
            insert_main_query
                .push("), tag_id FROM (SELECT tag_id FROM entity_tag WHERE tag_name IN (");
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

#[derive(Debug, sqlx::FromRow)]
struct ExportItem {
    entity_id: String,

    file_id: String,
    entity_type: EntityType,

    tags: String,

    count: i64,
    last_used: i64,
    created_at: i64,
}

pub async fn export_botimport(db: &DbConn, user_id: String) -> HandlerResult<Vec<u8>> {
    log::debug!("Exporting for user {}", user_id);

    let results: Vec<ExportItem> = sqlx::query_as(
        "SELECT entity_data.entity_id, entity_file.file_id, entity_file.entity_type, group_concat(entity_tag.tag_name, \" \") as tags, entity_data.count, entity_data.last_used, entity_data.created_at FROM entity_main \
        JOIN entity_data ON entity_data.combo_id = entity_main.combo_id \
        JOIN entity_tag ON entity_tag.tag_id = entity_main.tag_id \
        JOIN entity_file ON entity_file.entity_id = entity_data.entity_id \
        WHERE entity_data.user_id = $1 \
        GROUP BY entity_data.entity_id"
    ).bind(&user_id).fetch_all(db).await?;

    let import: BotImport = results
        .into_iter()
        .map(|item| ImportItem {
            entity_id: item.entity_id,
            file_id: item.file_id,
            entity_type: item.entity_type,
            tags: item.tags.split(" ").map(|s| s.to_string()).collect(),
            count: item.count,
            last_used: item.last_used,
            created_at: item.created_at,
        })
        .collect();

    log::debug!("Exported {} items", import.len());

    let json = serde_json::to_vec(&import)?;

    let mut compressor = GzEncoder::new(Vec::new(), Compression::best());
    compressor.write_all(json.as_slice())?;
    let compressed = compressor.finish()?;

    Ok(compressed)
}
