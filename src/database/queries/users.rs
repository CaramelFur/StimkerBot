use anyhow::Result;

use crate::types::DbConn;

pub async fn get_last_fix_time(db: &DbConn, user_id: String) -> Result<i64> {
  log::debug!("get_last_fix_time for user_id: {:?}", user_id);

  let result: Option<i64> = sqlx::query_scalar(
    "SELECT last_fixed_time FROM user_data \
      WHERE user_id = $1",
  )
  .bind(user_id)
  .fetch_optional(db)
  .await?;

  log::debug!("get_last_fix_time result: {:?}", result);

  Ok(result.unwrap_or(0))
}

pub async fn set_last_fix_time(db: &DbConn, user_id: String, time: i64) -> Result<()> {
  log::debug!(
    "set_last_fix_time for user_id: {:?} and time: {:?}",
    user_id,
    time
  );

  sqlx::query(
    "INSERT INTO user_data (user_id, last_fixed_time) \
      VALUES ($1, $2) \
      ON CONFLICT (user_id) DO UPDATE SET last_fixed_time = $2",
  )
  .bind(user_id.to_owned())
  .bind(time)
  .execute(db)
  .await?;

  log::debug!(
    "set_last_fix_time for user_id: {:?} and time: {:?} done",
    user_id,
    time
  );

  Ok(())
}

pub async fn wipe_user(db: &DbConn, user_id: String) -> Result<()> {
  log::debug!("wipe_user for user_id: {:?}", user_id);

  let mut transaction = db.begin().await?;

  sqlx::query(
    "DELETE FROM entity_main \
      WHERE combo_id IN \
      (SELECT combo_id FROM entity_data WHERE user_id = $1)",
  )
  .bind(user_id.clone())
  .execute(transaction.as_mut())
  .await?;

  sqlx::query(
    "DELETE FROM entity_data \
      WHERE user_id = $1",
  )
  .bind(user_id.clone())
  .execute(transaction.as_mut())
  .await?;

  sqlx::query(
    "DELETE FROM user_data \
      WHERE user_id = $1",
  )
  .bind(user_id.clone())
  .execute(transaction.as_mut())
  .await?;

  transaction.commit().await?;

  Ok(())
}
