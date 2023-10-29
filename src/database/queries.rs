use sqlx::{Error, QueryBuilder};

use crate::database::entities::{StickerStat, StickerTag};
use crate::types::DbConn;
use crate::util;

pub async fn insert_tags(
    db: &DbConn,
    user_id: String,
    sticker_id: String,
    file_id: String,
    tag_names: Vec<String>,
) -> Result<(), Error> {
    if tag_names.len() == 0 {
        return Ok(());
    }

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

    let mut query_builder = QueryBuilder::new(
        "INSERT OR IGNORE INTO sticker_tag (sticker_id, file_id, user_id, tag_name) ",
    );
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

    sqlx::query(
        "DELETE FROM sticker_tag \
        WHERE sticker_id = $1 AND user_id = $2",
    )
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
    if tag_names.len() == 0 {
        return Ok(());
    }

    log::debug!(
        "remove_tags: {:?} for sticker_id: {:?} and user_id: {:?}",
        tag_names,
        sticker_id,
        user_id
    );

    let mut query_builder = QueryBuilder::new(
        "DELETE FROM sticker_tag \
        WHERE sticker_id = ",
    );
    query_builder.push_bind(sticker_id);
    query_builder.push(" AND user_id = ");
    query_builder.push_bind(user_id);
    query_builder.push(" AND tag_name IN (");
    let mut seperator = query_builder.separated(", ");
    tag_names.iter().for_each(|tag_name| {
        seperator.push_bind(tag_name);
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

    let temp_result: Vec<(String,)> = sqlx::query_as(
        "SELECT tag_name FROM sticker_tag \
            WHERE sticker_id = $1 AND user_id = $2",
    )
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

    let mut query_builder = QueryBuilder::new(
        "SELECT sticker_tag.* FROM sticker_tag \
        LEFT JOIN sticker_stat ON sticker_tag.sticker_id = sticker_stat.sticker_id",
    );
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
        "SELECT sticker_tag.* FROM sticker_tag \
        LEFT JOIN sticker_stat ON sticker_tag.sticker_id = sticker_stat.sticker_id \
        WHERE sticker_tag.user_id = $1 \
        GROUP BY sticker_tag.sticker_id \
        ORDER BY sticker_stat.count DESC \
        LIMIT 50",
    )
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
) -> Result<(), Error> {
    log::debug!(
        "increase_sticker_stat for user_id: {:?} and unique_sticker_id: {:?}",
        user_id,
        unique_sticker_id
    );

    sqlx::query(
        "INSERT INTO sticker_stat (user_id, sticker_id, count, last_used) \
        VALUES ($1, $2, 1, $3) \
        ON CONFLICT (user_id, sticker_id) DO UPDATE SET count = sticker_stat.count + 1, last_used = $3"
    )
    .bind(user_id.clone())
    .bind(unique_sticker_id.clone())
    .bind(util::get_unix())
    .execute(db)
    .await?;

    log::debug!(
        "increase_sticker_stat for user_id: {:?} and unique_sticker_id: {:?} done",
        user_id,
        unique_sticker_id
    );

    Ok(())
}

pub async fn get_sticker_usage(
    db: &DbConn,
    user_id: String,
    unique_sticker_id: String,
) -> Result<StickerStat, Error> {
    log::debug!(
        "get_sticker_usage for user_id: {:?} and unique_sticker_id: {:?}",
        user_id,
        unique_sticker_id
    );

    let result: Option<StickerStat> = sqlx::query_as(
        "SELECT * FROM sticker_stat \
        WHERE user_id = $1 AND sticker_id = $2",
    )
    .bind(user_id.clone())
    .bind(unique_sticker_id.clone())
    .fetch_optional(db)
    .await?;

    log::debug!(
        "get_sticker_usage for user_id: {:?} and unique_sticker_id: {:?} done",
        user_id,
        unique_sticker_id
    );

    Ok(result.unwrap_or(StickerStat {
        user_id: user_id.clone(),
        sticker_id: unique_sticker_id.clone(),
        count: 0,
        last_used: 0,
    }))
}
