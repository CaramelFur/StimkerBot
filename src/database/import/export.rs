use std::io::Write;

use flate2::{write::GzEncoder, Compression};

use crate::{
    database::import::types::{BotImport, ExportItem, ImportItem},
    types::{DbConn, HandlerResult},
};

pub async fn export(db: &DbConn, user_id: String) -> HandlerResult<Vec<u8>> {
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
