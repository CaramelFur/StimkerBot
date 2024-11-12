#[derive(Debug, Default, Clone)]
pub struct InsertEntity {
  pub entity_id: String, // unique_id
  pub file_id: String,   // id
}

#[derive(Debug, Clone)]
pub struct GlobalStats {
  pub total_users: i64,
  pub total_tags: i64,

  pub total_stickers: i64,
  pub total_animations: i64,
  pub total_videos: i64,
  pub total_photos: i64,

  pub total_entities_sent: i64,
}
