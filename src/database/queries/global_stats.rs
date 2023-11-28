use anyhow::Result;

use crate::{database::{EntityType, queries::GlobalStats}, types::DbConn};

pub async fn get_global_stats(db: &DbConn) -> Result<GlobalStats> {
  log::debug!("get_global_stats");

  let total_users: i64 = sqlx::query_scalar("SELECT COUNT(DISTINCT user_id) FROM entity_data")
      .fetch_one(db)
      .await?;

  // count rows in entity_file
  let (total_stickers, total_animations, total_videos, total_photos): (i64, i64, i64, i64) =
      sqlx::query_as(
          "SELECT \
      (SELECT COUNT(*) FROM entity_file WHERE entity_type = $1) AS total_stickers, \
      (SELECT COUNT(*) FROM entity_file WHERE entity_type = $2) AS total_animations, \
      (SELECT COUNT(*) FROM entity_file WHERE entity_type = $3) AS total_videos, \
      (SELECT COUNT(*) FROM entity_file WHERE entity_type = $4) AS total_photos",
      )
      .bind(EntityType::Sticker)
      .bind(EntityType::Animation)
      .bind(EntityType::Video)
      .bind(EntityType::Photo)
      .fetch_one(db)
      .await?;

  let total_tags: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM entity_tag")
      .fetch_one(db)
      .await?;

  let total_entities_sent: i64 = sqlx::query_scalar("SELECT SUM(count) FROM entity_data")
      .fetch_one(db)
      .await?;

  let result = GlobalStats {
      total_users,
      total_stickers,
      total_animations,
      total_videos,
      total_photos,
      total_tags,
      total_entities_sent,
  };

  log::debug!("get_global_stats result: {:?}", result);

  Ok(result)
}
