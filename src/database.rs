use entity::sticker_tag;
use migration::OnConflict;
use sea_orm::DbBackend;
use sea_orm::QueryTrait;
use sea_orm::entity::prelude::*;
use sea_orm::DatabaseConnection;
use sea_orm::QuerySelect;
use sea_orm::Set;

pub async fn insert_tags(
    db: &DatabaseConnection,
    user_id: String,
    sticker_id: String,
    tag_names: Vec<String>,
) -> Result<(), DbErr> {
    log::debug!(
        "insert_tags: {:?} for sticker_id: {:?} and user_id: {:?}",
        tag_names,
        sticker_id,
        user_id
    );

    let new_tag_associations: Vec<sticker_tag::ActiveModel> = tag_names
        .iter()
        .map(|tag_name| sticker_tag::Model {
            sticker_id: sticker_id.clone(),
            user_id: user_id.clone(),
            tag_name: tag_name.to_owned(),
        }.into())
        .collect();

    log::debug!("new_tag_associations: {:?}", new_tag_associations);

    sticker_tag::Entity::insert_many(new_tag_associations)
        .on_conflict(OnConflict::new().do_nothing().to_owned())
        .do_nothing()
        .exec(db)
        .await?;

    Ok(())
}

pub async fn wipe_tags(
  db: &DatabaseConnection,
  user_id: String,
  sticker_id: String,
) -> Result<(), DbErr> {
    log::debug!(
        "wipe_tags for sticker_id: {:?} and user_id: {:?}",
        sticker_id,
        user_id
    );

    sticker_tag::Entity::delete_many()
        .filter(sticker_tag::Column::StickerId.eq(sticker_id))
        .filter(sticker_tag::Column::UserId.eq(user_id))
        .exec(db)
        .await?;

    Ok(())
}

pub async fn find_stickers(
    db: &DatabaseConnection,
    user_id: String,
    tags: Vec<String>,
) -> Result<Vec<String>, DbErr> {
    log::debug!(
        "find_stickers: {:?} for user_id: {:?}",
        tags,
        user_id
    );

    let query = sticker_tag::Entity::find()
        .filter(sticker_tag::Column::UserId.contains(user_id))
        .filter(sticker_tag::Column::TagName.is_in(tags.iter()))
        .group_by(sticker_tag::Column::StickerId)
        .having(Expr::expr(sticker_tag::Column::TagName.count()).gte(tags.len() as i32));

    let result = query
        .all(db)
        .await?
        .into_iter()
        .map(|sticker_tag| sticker_tag.sticker_id)
        .collect();

    log::debug!("find_stickers result: {:?}", result);

    Ok(result)
}
