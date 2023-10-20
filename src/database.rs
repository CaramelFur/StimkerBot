use entity::sticker_tag;
use migration::OnConflict;
use sea_orm::DbBackend;
use sea_orm::QueryTrait;
use sea_orm::entity::prelude::*;
use sea_orm::DatabaseConnection;
use sea_orm::QuerySelect;
use sea_orm::Set;

pub async fn insert_tag(
    db: &DatabaseConnection,
    user_id: String,
    sticker_id: String,
    tag_name: String,
) -> Result<(), DbErr> {
    log::debug!(
        "insert_tag: {:?} for sticker_id: {:?} and user_id: {:?}",
        tag_name,
        sticker_id,
        user_id
    );

    let new_tag_association: sticker_tag::ActiveModel = sticker_tag::Model {
        sticker_id,
        user_id,
        tag_name,
    }
    .into();

    log::debug!("new_tag_association: {:?}", new_tag_association);

    sticker_tag::Entity::insert(new_tag_association)
        .on_conflict(OnConflict::new().do_nothing().to_owned())
        .do_nothing()
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
