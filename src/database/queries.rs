use sqlx::{Error, QueryBuilder, Sqlite};

use crate::database::entities::{Entity, EntityStat, EntityType};
use crate::types::{DbConn, EntitySort};
use crate::util;

#[derive(Debug, Default, Clone)]
pub struct InsertEntity {
    pub entity_id: String,
    pub file_id: String,
}

pub async fn insert_tags(
    db: &DbConn,
    user_id: String,
    entities: Vec<InsertEntity>,
    entity_type: EntityType,
    tag_names: Vec<String>,
) -> Result<(), Error> {
    if tag_names.len() == 0 || entities.len() == 0 {
        return Ok(());
    }

    log::debug!(
        "insert_tags: {:?} for enties: {:?} and user_id: {:?}",
        tag_names,
        entities,
        user_id
    );

    let mut transaction = db.begin().await?;

    log::debug!("insert_tags: transaction started");

    // Insert the tag into the tag table if it doesn't exist
    let mut insert_tag_query = QueryBuilder::new("INSERT OR IGNORE INTO entity_tag (tag_name) ");
    insert_tag_query.push_values(tag_names.clone(), |mut b, tag_name| {
        b.push_bind(tag_name);
    });
    insert_tag_query
        .build()
        .execute(transaction.as_mut())
        .await?;

    log::debug!("insert_tags: tag inserted");

    // Insert the file into the entity table if it doesn't exist
    let mut insert_file_query =
        QueryBuilder::new("INSERT OR REPLACE INTO entity_file (entity_id, file_id, entity_type) ");
    insert_file_query.push_values(&entities, |mut b, entity| {
        b.push_bind(entity.entity_id.clone());
        b.push_bind(entity.file_id.clone());
        b.push_bind(entity_type.clone());
    });
    insert_file_query
        .build()
        .execute(transaction.as_mut())
        .await?;

    log::debug!("insert_tags: file inserted");

    for entity in entities {
        // Insert a relation between the entity and the tag
        let mut insert_main_query: QueryBuilder<'_, Sqlite> =
            QueryBuilder::new("INSERT OR IGNORE INTO entity_main (entity_id, user_id, tag_id) ");
        insert_main_query.push_values(tag_names.clone(), |mut b, tag_name| {
            b.push_bind(entity.entity_id.clone());
            b.push_bind(user_id.clone());
            b.push("(SELECT tag_id from entity_tag where tag_name = ")
                .push_bind_unseparated(tag_name)
                .push_unseparated(")");
        });
        insert_main_query
            .build()
            .execute(transaction.as_mut())
            .await?;
    }

    log::debug!("insert_tags: main inserted");

    transaction.commit().await?;

    log::debug!("insert_tags: transaction committed");

    Ok(())
}

pub async fn remove_tags(
    db: &DbConn,
    user_id: String,
    entity_ids: Vec<String>,
    tag_names: Vec<String>,
) -> Result<(), Error> {
    if tag_names.len() == 0 || entity_ids.len() == 0 {
        return Ok(());
    }

    log::debug!(
        "remove_tags: {:?} for entity_ids: {:?} and user_id: {:?}",
        tag_names,
        entity_ids,
        user_id
    );

    let mut query_builder = QueryBuilder::new(
        "DELETE FROM entity_main \
        WHERE user_id = ",
    );
    query_builder.push_bind(user_id);
    query_builder.push(" AND entity_id IN (");
    let mut seperator = query_builder.separated(", ");
    entity_ids.iter().for_each(|entity_id| {
        seperator.push_bind(entity_id);
    });
    query_builder.push(")");
    query_builder.push(" AND tag_id IN (");
    query_builder.push("SELECT tag_id FROM entity_tag WHERE tag_name IN (");
    seperator = query_builder.separated(", ");
    tag_names.iter().for_each(|tag_name| {
        seperator.push_bind(tag_name);
    });
    query_builder.push(")");
    query_builder.push(")");

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
        "DELETE FROM entity_main \
        WHERE entity_id = $1 AND user_id = $2",
    )
    .bind(entity_id)
    .bind(user_id)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn wipe_user(db: &DbConn, user_id: String) -> Result<(), Error> {
    log::debug!("wipe_user for user_id: {:?}", user_id);

    sqlx::query(
        "DELETE FROM entity_main \
        WHERE user_id = $1",
    )
    .bind(user_id)
    .execute(db)
    .await?;

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
        "SELECT tag_name FROM entity_main \
        JOIN entity_tag ON entity_tag.tag_id = entity_main.tag_id \
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

/// Search ===============================

pub async fn find_entities(
    db: &DbConn,
    user_id: String,
    tags: Vec<String>,
    page: i32,
    sort: EntitySort,
) -> Result<Vec<Entity>, Error> {
    log::debug!("find_entities: {:?} for user_id: {:?}", tags, user_id);

    let mut query_builder = QueryBuilder::new(
        "SELECT entity_main.entity_id, entity_main.user_id, entity_file.file_id, entity_file.entity_type FROM entity_main \
        LEFT JOIN entity_stat ON entity_main.entity_id = entity_stat.entity_id \
        JOIN entity_tag ON entity_tag.tag_id = entity_main.tag_id \
        JOIN entity_file ON entity_file.entity_id = entity_main.entity_id",
    );
    query_builder.push(" WHERE entity_main.user_id = ");
    query_builder.push_bind(user_id);
    query_builder.push(" AND entity_tag.tag_name IN (");
    query_builder.push_values(tags.iter(), |mut b, tag_name| {
        b.push_bind(tag_name);
    });
    query_builder.push(")");
    query_builder.push(" GROUP BY entity_main.entity_id");
    query_builder.push(" HAVING COUNT(entity_tag.tag_name) >= ");
    query_builder.push_bind(tags.len() as i32);
    query_builder.push(" ORDER BY entity_stat.count DESC");
    query_builder.push(" LIMIT 50");

    let result: Vec<Entity> = query_builder.build_query_as().fetch_all(db).await?;

    log::debug!("find_entities result: {:?}", result);

    Ok(result)
}

pub async fn list_entities(
    db: &DbConn,
    user_id: String,
    page: i32,
    sort: EntitySort,
) -> Result<Vec<Entity>, Error> {
    log::debug!("list_entities for user_id: {:?}", user_id);

    let result: Vec<Entity> = sqlx::query_as(
        "SELECT entity_main.entity_id, entity_main.user_id, entity_file.file_id, entity_file.entity_type FROM entity_main \
        LEFT JOIN entity_stat ON entity_main.entity_id = entity_stat.entity_id \
        JOIN entity_file ON entity_file.entity_id = entity_main.entity_id \
        WHERE entity_main.user_id = $1 \
        GROUP BY entity_main.entity_id \
        ORDER BY entity_stat.count DESC \
        LIMIT 50",
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;

    log::debug!("list_entities result: {:?}", result);

    Ok(result)
}

/// Stats ===============================

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
