use crate::{database::EntityStat, types::DbConn, util};
use anyhow::Result;

pub async fn increase_entity_stat(
  db: &DbConn,
  user_id: String,
  unique_entity_id: String,
) -> Result<()> {
  log::debug!(
    "increase_entity_stat for user_id: {:?} and unique_entity_id: {:?}",
    user_id,
    unique_entity_id
  );

  sqlx::query(
    "INSERT INTO entity_data (user_id, entity_id, count, last_used) \
      VALUES ($1, $2, 1, $3) \
      ON CONFLICT (user_id, entity_id) DO UPDATE SET count = entity_data.count + 1, last_used = $3",
  )
  .bind(user_id.clone())
  .bind(unique_entity_id.clone())
  .bind(util::get_unix())
  .execute(db)
  .await?;

  log::debug!(
    "increase_entity_stat for user_id: {:?} and unique_entity_id: {:?} done",
    user_id,
    unique_entity_id
  );

  Ok(())
}

pub async fn get_entity_usage(
  db: &DbConn,
  user_id: String,
  unique_entity_id: String,
) -> Result<EntityStat> {
  log::debug!(
    "get_entity_usage for user_id: {:?} and unique_entity_id: {:?}",
    user_id,
    unique_entity_id
  );

  let result: Option<EntityStat> = sqlx::query_as(
    "SELECT * FROM entity_data \
      WHERE user_id = $1 AND entity_id = $2",
  )
  .bind(user_id.clone())
  .bind(unique_entity_id.clone())
  .fetch_optional(db)
  .await?;

  log::debug!(
    "get_entity_usage for user_id: {:?} and unique_entity_id: {:?} done",
    user_id,
    unique_entity_id
  );

  Ok(result.unwrap_or(EntityStat {
    user_id: user_id.clone(),
    entity_id: unique_entity_id.clone(),
    count: 0,
    last_used: 0,
    created_at: 0,
  }))
}
