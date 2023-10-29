use sqlx::{Error, QueryBuilder};

use crate::database::entities::{EntityStat, EntityTag};
use crate::types::DbConn;
use crate::util;

pub async fn insert_tags(
    db: &DbConn,
    user_id: String,
    entity_id: String,
    file_id: String,
    tag_names: Vec<String>,
) -> Result<(), Error> {
    if tag_names.len() == 0 {
        return Ok(());
    }

    log::debug!(
        "insert_tags: {:?} for entity_id: {:?} and user_id: {:?}",
        tag_names,
        entity_id,
        user_id
    );

    let new_tag_associations: Vec<EntityTag> = tag_names
        .iter()
        .map(|tag_name| EntityTag {
            entity_id: entity_id.to_owned(),
            file_id: file_id.to_owned(),
            user_id: user_id.to_owned(),
            tag_name: tag_name.to_owned(),
        })
        .collect();

    log::debug!("new_tag_associations: {:?}", new_tag_associations);

    let mut query_builder = QueryBuilder::new(
        "INSERT OR IGNORE INTO entity_tag (entity_id, file_id, user_id, tag_name) ",
    );
    query_builder.push_values(new_tag_associations, |mut b, new_category| {
        b.push_bind(new_category.entity_id)
            .push_bind(new_category.file_id)
            .push_bind(new_category.user_id)
            .push_bind(new_category.tag_name);
    });

    query_builder.build().execute(db).await?;

    Ok(())
}

pub async fn wipe_tags(db: &DbConn, user_id: String, entity_id: String) -> Result<(), Error> {
    log::debug!(
        "wipe_tags for entity_id: {:?} and user_id: {:?}",
        entity_id,
        user_id
    );

    sqlx::query(
        "DELETE FROM entity_tag \
        WHERE entity_id = $1 AND user_id = $2",
    )
    .bind(entity_id)
    .bind(user_id)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn remove_tags(
    db: &DbConn,
    user_id: String,
    entity_id: String,
    tag_names: Vec<String>,
) -> Result<(), Error> {
    if tag_names.len() == 0 {
        return Ok(());
    }

    log::debug!(
        "remove_tags: {:?} for entity_id: {:?} and user_id: {:?}",
        tag_names,
        entity_id,
        user_id
    );

    let mut query_builder = QueryBuilder::new(
        "DELETE FROM entity_tag \
        WHERE entity_id = ",
    );
    query_builder.push_bind(entity_id);
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
    entity_id: String,
) -> Result<Vec<String>, Error> {
    log::debug!(
        "get_tags for entity_id: {:?} and user_id: {:?}",
        entity_id,
        user_id
    );

    let temp_result: Vec<(String,)> = sqlx::query_as(
        "SELECT tag_name FROM entity_tag \
            WHERE entity_id = $1 AND user_id = $2",
    )
    .bind(entity_id)
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

pub async fn find_entities(
    db: &DbConn,
    user_id: String,
    tags: Vec<String>,
) -> Result<Vec<EntityTag>, Error> {
    log::debug!("find_entities: {:?} for user_id: {:?}", tags, user_id);

    let mut query_builder = QueryBuilder::new(
        "SELECT entity_tag.* FROM entity_tag \
        LEFT JOIN entity_stat ON entity_tag.entity_id = entity_stat.entity_id",
    );
    query_builder.push(" WHERE entity_tag.user_id = ");
    query_builder.push_bind(user_id);
    query_builder.push(" AND entity_tag.tag_name IN (");
    query_builder.push_values(tags.iter(), |mut b, tag_name| {
        b.push_bind(tag_name);
    });
    query_builder.push(")");
    query_builder.push(" GROUP BY entity_tag.entity_id");
    query_builder.push(" HAVING COUNT(entity_tag.tag_name) >= ");
    query_builder.push_bind(tags.len() as i32);
    query_builder.push(" ORDER BY entity_stat.count DESC");
    query_builder.push(" LIMIT 50");

    let result: Vec<EntityTag> = query_builder.build_query_as().fetch_all(db).await?;

    log::debug!("find_entities result: {:?}", result);

    Ok(result)
}

pub async fn list_entities(db: &DbConn, user_id: String) -> Result<Vec<EntityTag>, Error> {
    log::debug!("list_entities for user_id: {:?}", user_id);

    let result: Vec<EntityTag> = sqlx::query_as(
        "SELECT entity_tag.* FROM entity_tag \
        LEFT JOIN entity_stat ON entity_tag.entity_id = entity_stat.entity_id \
        WHERE entity_tag.user_id = $1 \
        GROUP BY entity_tag.entity_id \
        ORDER BY entity_stat.count DESC \
        LIMIT 50",
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;

    log::debug!("list_entities result: {:?}", result);

    Ok(result)
}

pub async fn increase_entity_stat(
    db: &DbConn,
    user_id: String,
    unique_entity_id: String,
) -> Result<(), Error> {
    log::debug!(
        "increase_entity_stat for user_id: {:?} and unique_entity_id: {:?}",
        user_id,
        unique_entity_id
    );

    sqlx::query(
        "INSERT INTO entity_stat (user_id, entity_id, count, last_used) \
        VALUES ($1, $2, 1, $3) \
        ON CONFLICT (user_id, entity_id) DO UPDATE SET count = entity_stat.count + 1, last_used = $3"
    )
    .bind(user_id.clone())
    .bind(unique_entity_id.clone())
    .bind(util::get_unix())
    .execute(db)
    .await?;

    log::debug!(
        "increase_entity_stat for user_id: {:?} and unique_entity_id: {:?} done",
        user_id,
        unique_entity_id
    );

    Ok(())
}

pub async fn get_entity_usage(
    db: &DbConn,
    user_id: String,
    unique_entity_id: String,
) -> Result<EntityStat, Error> {
    log::debug!(
        "get_entity_usage for user_id: {:?} and unique_entity_id: {:?}",
        user_id,
        unique_entity_id
    );

    let result: Option<EntityStat> = sqlx::query_as(
        "SELECT * FROM entity_stat \
        WHERE user_id = $1 AND entity_id = $2",
    )
    .bind(user_id.clone())
    .bind(unique_entity_id.clone())
    .fetch_optional(db)
    .await?;

    log::debug!(
        "get_entity_usage for user_id: {:?} and unique_entity_id: {:?} done",
        user_id,
        unique_entity_id
    );

    Ok(result.unwrap_or(EntityStat {
        user_id: user_id.clone(),
        entity_id: unique_entity_id.clone(),
        count: 0,
        last_used: 0,
    }))
}
