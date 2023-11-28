use anyhow::Result;
use sqlx::{QueryBuilder, Sqlite};

use crate::{database::EntityType, types::DbConn, util};

use super::InsertEntity;

pub async fn insert_tags(
    db: &DbConn,
    user_id: String,
    entities: Vec<InsertEntity>,
    entity_type: EntityType,
    tag_names: Vec<String>,
) -> Result<()> {
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

    // Insert the user/entity combo
    let mut insert_ue_query =
        QueryBuilder::new("INSERT OR IGNORE INTO entity_data (entity_id, user_id, created_at) ");
    insert_ue_query.push_values(&entities, |mut b, entity| {
        b.push_bind(entity.entity_id.clone());
        b.push_bind(user_id.clone());
        b.push_bind(util::get_unix());
    });
    insert_ue_query
        .build()
        .execute(transaction.as_mut())
        .await?;

    for entity in entities {
        // Insert a relation between the entity and the tag
        let mut insert_main_query: QueryBuilder<'_, Sqlite> = QueryBuilder::new(
            "INSERT OR IGNORE INTO entity_main (combo_id, tag_id) \
              SELECT (SELECT combo_id FROM entity_data WHERE entity_id = ",
        );
        insert_main_query.push_bind(&entity.entity_id);
        insert_main_query.push(" AND user_id = ");
        insert_main_query.push_bind(&user_id);
        insert_main_query.push("), tag_id FROM (SELECT tag_id FROM entity_tag WHERE tag_name IN (");
        let mut seperator = insert_main_query.separated(", ");
        tag_names.iter().for_each(|tag_name| {
            seperator.push_bind(tag_name);
        });
        insert_main_query.push("))");

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
) -> Result<()> {
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
      WHERE combo_id IN (",
    );
    query_builder.push("SELECT combo_id FROM entity_data WHERE user_id = ");
    query_builder.push_bind(user_id);
    query_builder.push(" AND entity_id IN (");
    let mut seperator = query_builder.separated(", ");
    entity_ids.into_iter().for_each(|entity_id| {
        seperator.push_bind(entity_id);
    });
    query_builder.push("))");
    query_builder.push(" AND tag_id IN (");
    query_builder.push("SELECT tag_id FROM entity_tag WHERE tag_name IN (");
    seperator = query_builder.separated(", ");
    tag_names.into_iter().for_each(|tag_name| {
        seperator.push_bind(tag_name);
    });
    query_builder.push("))");

    query_builder.build().execute(db).await?;

    Ok(())
}

pub async fn get_tags(db: &DbConn, user_id: String, entity_id: String) -> Result<Vec<String>> {
    log::debug!(
        "get_tags for entity_id: {:?} and user_id: {:?}",
        entity_id,
        user_id
    );

    let temp_result: Vec<(String,)> = sqlx::query_as(
        "SELECT tag_name FROM entity_main \
      JOIN entity_tag ON entity_tag.tag_id = entity_main.tag_id \
      JOIN entity_data ON entity_data.combo_id = entity_main.combo_id \
      WHERE entity_data.entity_id = $1 AND entity_data.user_id = $2",
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

pub async fn wipe_tags(db: &DbConn, user_id: String, entity_id: String) -> Result<()> {
    log::debug!(
        "wipe_tags for entity_id: {:?} and user_id: {:?}",
        entity_id,
        user_id
    );

    sqlx::query(
        "DELETE FROM entity_main \
      WHERE combo_id IN \
      (SELECT combo_id FROM entity_data WHERE entity_id = $1 AND user_id = $2)",
    )
    .bind(entity_id)
    .bind(user_id)
    .execute(db)
    .await?;

    Ok(())
}
