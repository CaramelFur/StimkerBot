use entity::sticker_stat;
use entity::sticker_tag;
use migration::OnConflict;
use sea_orm::entity::prelude::*;
use sea_orm::DatabaseConnection;
use sea_orm::QueryOrder;
use sea_orm::QuerySelect;

use crate::util;

pub async fn insert_tags(
    db: &DatabaseConnection,
    user_id: String,
    unique_sticker_id: String,
    file_id: String,
    tag_names: Vec<String>,
) -> Result<(), DbErr> {
    log::debug!(
        "insert_tags: {:?} for sticker_id: {:?} and user_id: {:?}",
        tag_names,
        unique_sticker_id,
        user_id
    );

    let new_tag_associations: Vec<sticker_tag::ActiveModel> = tag_names
        .iter()
        .map(|tag_name| {
            sticker_tag::Model {
                sticker_id: unique_sticker_id.to_owned(),
                file_id: file_id.to_owned(),
                user_id: user_id.to_owned(),
                tag_name: tag_name.to_owned(),
            }
            .into()
        })
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
    unique_sticker_id: String,
) -> Result<(), DbErr> {
    log::debug!(
        "wipe_tags for sticker_id: {:?} and user_id: {:?}",
        unique_sticker_id,
        user_id
    );

    sticker_tag::Entity::delete_many()
        .filter(sticker_tag::Column::StickerId.eq(unique_sticker_id))
        .filter(sticker_tag::Column::UserId.eq(user_id))
        .exec(db)
        .await?;

    Ok(())
}

pub async fn remove_tags(
    db: &DatabaseConnection,
    user_id: String,
    unique_sticker_id: String,
    tag_names: Vec<String>,
) -> Result<(), DbErr> {
    log::debug!(
        "remove_tags: {:?} for sticker_id: {:?} and user_id: {:?}",
        tag_names,
        unique_sticker_id,
        user_id
    );

    sticker_tag::Entity::delete_many()
        .filter(sticker_tag::Column::StickerId.eq(unique_sticker_id))
        .filter(sticker_tag::Column::UserId.eq(user_id))
        .filter(sticker_tag::Column::TagName.is_in(tag_names.iter()))
        .exec(db)
        .await?;

    Ok(())
}

pub async fn get_tags(
    db: &DatabaseConnection,
    user_id: String,
    unique_sticker_id: String,
) -> Result<Vec<String>, DbErr> {
    log::debug!(
        "get_tags for sticker_id: {:?} and user_id: {:?}",
        unique_sticker_id,
        user_id
    );

    let query = sticker_tag::Entity::find()
        .filter(sticker_tag::Column::StickerId.eq(unique_sticker_id))
        .filter(sticker_tag::Column::UserId.eq(user_id));

    let result = query
        .all(db)
        .await?
        .into_iter()
        .map(|sticker_tag| sticker_tag.tag_name)
        .collect();

    log::debug!("get_tags result: {:?}", result);

    Ok(result)
}

pub async fn find_stickers(
    db: &DatabaseConnection,
    user_id: String,
    tags: Vec<String>,
) -> Result<Vec<sticker_tag::Model>, DbErr> {
    log::debug!("find_stickers: {:?} for user_id: {:?}", tags, user_id);

    let query = sticker_tag::Entity::find()
        .filter(sticker_tag::Column::UserId.contains(user_id))
        .filter(sticker_tag::Column::TagName.is_in(tags.iter()))
        .group_by(sticker_tag::Column::StickerId)
        .having(Expr::expr(sticker_tag::Column::TagName.count()).gte(tags.len() as i32))
        .left_join(sticker_stat::Entity)
        .order_by(sticker_stat::Column::Count, sea_orm::Order::Desc)
        .limit(50);

    let result = query.all(db).await?;

    log::debug!("find_stickers result: {:?}", result);

    Ok(result)
}

pub async fn list_stickers(
    db: &DatabaseConnection,
    user_id: String,
) -> Result<Vec<sticker_tag::Model>, DbErr> {
    log::debug!("list_stickers for user_id: {:?}", user_id);

    let query = sticker_tag::Entity::find()
        .filter(sticker_tag::Column::UserId.contains(user_id))
        .group_by(sticker_tag::Column::StickerId)
        .left_join(sticker_stat::Entity)
        .order_by(sticker_stat::Column::Count, sea_orm::Order::Desc)
        .limit(50);

    let result = query.all(db).await?;

    log::debug!("list_stickers result: {:?}", result);

    Ok(result)
}

pub async fn increase_sticker_stat(
    db: &DatabaseConnection,
    user_id: String,
    unique_sticker_id: String,
) -> Result<(), DbErr> {
    log::debug!(
        "increase_sticker_stat for user_id: {:?} and unique_sticker_id: {:?}",
        user_id,
        unique_sticker_id
    );

    let model: sticker_stat::ActiveModel = sticker_stat::Model {
        user_id: user_id.to_owned(),
        sticker_id: unique_sticker_id.to_owned(),
        count: 1,
        last_used: util::get_unix(),
    }
    .into();

    sticker_stat::Entity::insert(model)
        .on_conflict(
            OnConflict::new()
                .value(
                    sticker_stat::Column::Count,
                    Expr::col(sticker_stat::Column::Count).add(1),
                )
                .value(
                    sticker_stat::Column::LastUsed,
                    Expr::col(sticker_stat::Column::LastUsed).add(util::get_unix()),
                )
                .to_owned(),
        )
        .exec(db)
        .await?;

    log::debug!(
        "increase_sticker_stat for user_id: {:?} and unique_sticker_id: {:?} done",
        user_id,
        unique_sticker_id
    );

    Ok(())
}

pub struct StickerUsage {
    pub count: i64,
    pub last_used: i64,
}

pub async fn get_sticker_usage(
    db: &DatabaseConnection,
    user_id: String,
    unique_sticker_id: String,
) -> Result<Option<StickerUsage>, DbErr> {
    log::debug!(
        "get_sticker_usage for user_id: {:?} and unique_sticker_id: {:?}",
        user_id,
        unique_sticker_id
    );

    let query = sticker_stat::Entity::find()
        .filter(sticker_stat::Column::UserId.eq(&user_id))
        .filter(sticker_stat::Column::StickerId.eq(&unique_sticker_id));

    let result = query
        .one(db)
        .await?
        .map(|sticker_stat| StickerUsage {
            count: sticker_stat.count,
            last_used: sticker_stat.last_used,
        });

    log::debug!(
        "get_sticker_usage for user_id: {:?} and unique_sticker_id: {:?} done",
        user_id,
        unique_sticker_id
    );

    Ok(result)
}
