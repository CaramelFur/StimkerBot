use serde::{Deserialize, Serialize};

use crate::database::entities::EntityType;

pub type QSBotImport = Vec<QSBotImportItem>;

#[derive(Deserialize, Debug)]
pub struct QSBotImportItem {
  pub id: String,
  #[serde(rename = "fileId")]
  pub file_id: String,
  pub tags: Vec<String>,
  #[serde(rename = "set")]
  pub _set: Option<String>,
  #[serde(rename = "isAnimated")]
  pub _is_animated: Option<bool>,
}

pub type BotImport = Vec<ImportItem>;

#[derive(Deserialize, Serialize, Debug)]
pub struct ImportItem {
  #[serde(rename = "id")]
  pub entity_id: String,

  #[serde(rename = "fi")]
  pub file_id: String,
  #[serde(rename = "tp")]
  pub entity_type: EntityType,
  #[serde(rename = "t")]
  pub tags: Vec<String>,

  #[serde(rename = "c")]
  pub count: i64,
  #[serde(rename = "lu")]
  pub last_used: i64,
  #[serde(rename = "ca")]
  pub created_at: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ExportItem {
  pub entity_id: String,

  pub file_id: String,
  pub entity_type: EntityType,

  pub tags: String,

  pub count: i64,
  pub last_used: i64,
  pub created_at: i64,
}
