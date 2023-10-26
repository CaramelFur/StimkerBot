use sqlx::{Error, QueryBuilder};

use crate::database::entities::StickerTag;
use crate::types::DbConn;
use crate::util;

pub async fn insert_tags(
    db: &DbConn,
    user_id: String,
    sticker_id: String,
    file_id: String,
    tag_names: Vec<String>,
) -> Result<(), Error> {
    log::debug!(
        "insert_tags: {:?} for sticker_id: {:?} and user_id: {:?}",
        tag_names,
        sticker_id,
        user_id
    );

    let new_tag_associations: Vec<StickerTag> = tag_names
        .iter()
        .map(|tag_name| StickerTag {
            sticker_id: sticker_id.to_owned(),
            file_id: file_id.to_owned(),
            user_id: user_id.to_owned(),
            tag_name: tag_name.to_owned(),
        })
        .collect();

    log::debug!("new_tag_associations: {:?}", new_tag_associations);

    let mut query_builder =
        QueryBuilder::new("INSERT INTO sticker_tag (sticker_id, file_id, user_id, tag_name)");
    query_builder.push_values(new_tag_associations, |mut b, new_category| {
        b.push_bind(new_category.sticker_id)
            .push_bind(new_category.file_id)
            .push_bind(new_category.user_id)
            .push_bind(new_category.tag_name);
    });

    query_builder.build().execute(db).await?;

    Ok(())
}

pub async fn wipe_tags(db: &DbConn, user_id: String, sticker_id: String) -> Result<(), Error> {
    log::debug!(
        "wipe_tags for sticker_id: {:?} and user_id: {:?}",
        sticker_id,
        user_id
    );

    sqlx::query("DELETE FROM sticker_tag WHERE sticker_id = $1 AND user_id = $2")
        .bind(sticker_id)
        .bind(user_id)
        .execute(db)
        .await?;

    Ok(())
}

pub async fn remove_tags(
    db: &DbConn,
    user_id: String,
    sticker_id: String,
    tag_names: Vec<String>,
) -> Result<(), Error> {
    log::debug!(
        "remove_tags: {:?} for sticker_id: {:?} and user_id: {:?}",
        tag_names,
        sticker_id,
        user_id
    );

    let mut query_builder = QueryBuilder::new("DELETE FROM sticker_tag WHERE sticker_id = ");
    query_builder.push_bind(sticker_id);
    query_builder.push(" AND user_id = ");
    query_builder.push_bind(user_id);
    query_builder.push(" AND tag_name IN (");
    query_builder.push_values(tag_names.iter(), |mut b, tag_name| {
        b.push_bind(tag_name);
    });
    query_builder.push(")");

    query_builder.build().execute(db).await?;

    Ok(())
}

pub async fn get_tags(
    db: &DbConn,
    user_id: String,
    sticker_id: String,
) -> Result<Vec<String>, Error> {
    log::debug!(
        "get_tags for sticker_id: {:?} and user_id: {:?}",
        sticker_id,
        user_id
    );

    let temp_result: Vec<(String,)> =
        sqlx::query_as("SELECT tag_name FROM sticker_tag WHERE sticker_id = $1 AND user_id = $2")
            .bind(sticker_id)
            .bind(user_id)
            .fetch_all(db)
            .await?;

    let result: Vec<String> = temp_result
        .into_iter()
        .map(|(tag_name,)| tag_name)
        .collect();

    log::debug!("get_tags result: {:?}", result);

    Ok(result)
}

pub async fn find_stickers(
    db: &DbConn,
    user_id: String,
    tags: Vec<String>,
) -> Result<Vec<StickerTag>, Error> {
    log::debug!("find_stickers: {:?} for user_id: {:?}", tags, user_id);

    let mut query_builder = QueryBuilder::new("SELECT * FROM sticker_tag LEFT JOIN sticker_stat ON sticker_tag.sticker_id = sticker_stat.sticker_id");
    query_builder.push(" WHERE sticker_tag.user_id = ");
    query_builder.push_bind(user_id);
    query_builder.push(" AND sticker_tag.tag_name IN (");
    query_builder.push_values(tags.iter(), |mut b, tag_name| {
        b.push_bind(tag_name);
    });
    query_builder.push(")");
    query_builder.push(" GROUP BY sticker_tag.sticker_id");
    query_builder.push(" HAVING COUNT(sticker_tag.tag_name) >= ");
    query_builder.push_bind(tags.len() as i32);
    query_builder.push(" ORDER BY sticker_stat.count DESC");
    query_builder.push(" LIMIT 50");

    let result: Vec<StickerTag> = query_builder.build_query_as().fetch_all(db).await?;

    log::debug!("find_stickers result: {:?}", result);

    Ok(result)
}

pub async fn list_stickers(db: &DbConn, user_id: String) -> Result<Vec<StickerTag>, Error> {
    log::debug!("list_stickers for user_id: {:?}", user_id);

    let result: Vec<StickerTag> = sqlx::query_as(
        "SELECT * FROM sticker_tag LEFT JOIN sticker_stat ON sticker_tag.sticker_id = sticker_stat.sticker_id WHERE sticker_tag.user_id = $1 GROUP BY sticker_tag.sticker_id ORDER BY sticker_stat.count DESC LIMIT 50")
        .bind(user_id)
        .fetch_all(db)
        .await?;

    log::debug!("list_stickers result: {:?}", result);

    Ok(result)
}

pub async fn increase_sticker_stat(
    db: &DbConn,
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
    db: &DbConn,
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

    let result = query.one(db).await?.map(|sticker_stat| StickerUsage {
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
