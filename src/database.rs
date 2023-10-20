use entity::sticker_tag;
use entity::tag;

use sea_orm::entity::prelude::*;
use sea_orm::{DatabaseConnection, QuerySelect};

pub async fn insert_tag(
    db: &DatabaseConnection,
    user_id: String,
    sticker_id: String,
    tag: String,
) -> Result<(), DbErr> {
    let new_tag: sticker_tag::ActiveModel = sticker_tag::Model {
        sticker_id,
        user_id,
        tag,
    }
    .into();

    sticker_tag::Entity::insert(new_tag).exec(db).await?;

    Ok(())
}

pub async fn find_stickers(
  db: &DatabaseConnection,
  user_id: String,
  tags: Vec<String>,
) -> Result<Vec<String>, DbErr> {
  let mut query = sticker_tag::Entity::find();

  // return all sticker_ids where user_id is equal to user_id and the tag is equal to all tags


  Ok(stickers.into_iter().map(|sticker| sticker.sticker_id).collect())
}
