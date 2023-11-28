use anyhow::Result;

use crate::types::DbConn;

pub async fn update_file_id(db: &DbConn, entity_id: String, file_id: String) -> Result<()> {
    log::debug!(
        "update_file_id for entity_id: {:?} and file_id: {:?}",
        entity_id,
        file_id
    );

    sqlx::query("UPDATE entity_file SET file_id = $1 WHERE entity_id = $2")
        .bind(file_id)
        .bind(entity_id)
        .execute(db)
        .await?;

    Ok(())
}
