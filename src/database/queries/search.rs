use anyhow::Result;
use sqlx::QueryBuilder;

use crate::{
  database::Entity,
  types::{DbConn, EntitySort, InlineSearchQuery},
  util,
};

pub async fn find_entities(
  db: &DbConn,
  user_id: String,
  query: InlineSearchQuery,
  page: i32,
) -> Result<Vec<Entity>> {
  if query.get_all {
    return list_entities(db, user_id, query.sort, page).await;
  }

  log::debug!("find_entities: {:?} for user_id: {:?}", query.tags, user_id);

  let tags_len: i32 = query.tags.len() as i32;
  let negative_tags_len: i32 = query.negative_tags.len() as i32;

  if tags_len == 0 {
    log::warn!("find_entities called with empty tags");
    return Ok(vec![]);
  }

  let mut query_builder = QueryBuilder::new(
      "SELECT entity_data.entity_id, entity_data.user_id, entity_file.file_id, entity_file.entity_type FROM entity_main \
      JOIN entity_tag ON entity_tag.tag_id = entity_main.tag_id \
      JOIN entity_data ON entity_data.combo_id = entity_main.combo_id \
      JOIN entity_file ON entity_file.entity_id = entity_data.entity_id",
  );
  query_builder
    .push(" WHERE entity_data.user_id = ")
    .push_bind(user_id.to_owned());

  if let Some(entity_type) = query.entity_type {
    query_builder
      .push(" AND entity_file.entity_type = ")
      .push_bind(entity_type.to_owned());
  }

  if query.negative_tags.len() > 0 {
    query_builder
      .push(
        "AND entity_data.entity_id NOT IN ( \
              SELECT entity_data.entity_id FROM entity_main \
              JOIN entity_tag ON entity_tag.tag_id = entity_main.tag_id \
            JOIN entity_data ON entity_data.combo_id = entity_main.combo_id \
              WHERE entity_data.user_id = ",
      )
      .push_bind(user_id.to_owned())
      .push(" AND (");
    let mut seperator_builder = query_builder.separated("OR");
    query.negative_tags.into_iter().for_each(|tag_name| {
      let mut escaped_tag = tag_name.replace("_", "\\_").replace("%", "\\%");
      let needs_escape = escaped_tag != tag_name;
      escaped_tag += "%";

      seperator_builder
        .push(" entity_tag.tag_name LIKE ")
        .push_bind_unseparated(escaped_tag);

      if needs_escape {
        seperator_builder.push_unseparated(" ESCAPE '\\' ");
      }
    });
    query_builder
      .push(")")
      .push("GROUP BY entity_main.combo_id HAVING COUNT(entity_tag.tag_name) >= ")
      .push_bind(negative_tags_len)
      .push(")");
  }

  query_builder.push(" AND (");
  let mut seperator_builder = query_builder.separated("OR");
  query.tags.into_iter().for_each(|tag_name| {
    let mut escaped_tag = tag_name.replace("_", "\\_").replace("%", "\\%");
    let needs_escape = escaped_tag != tag_name;
    escaped_tag += "%";

    seperator_builder
      .push(" entity_tag.tag_name LIKE ")
      .push_bind_unseparated(escaped_tag);

    if needs_escape {
      seperator_builder.push_unseparated(" ESCAPE '\\' ");
    }
  });
  query_builder
    .push(")")
    .push(" GROUP BY entity_main.combo_id") // Since we filter by user, this is possible
    .push(" HAVING COUNT(entity_tag.tag_name) >= ")
    .push_bind(tags_len)
    .push(" ORDER BY ")
    .push(query.sort.to_sql())
    .push(" LIMIT 50 OFFSET ")
    .push_bind(page * 50);

  let start_time = util::get_unix();
  let result: Vec<Entity> = query_builder.build_query_as().fetch_all(db).await?;
  let end_time = util::get_unix();
  log::info!("Search query took {}ms", end_time - start_time);

  log::debug!("find_entities result: {:?}", result);

  Ok(result)
}

async fn list_entities(
  db: &DbConn,
  user_id: String,
  sort: EntitySort,
  page: i32,
) -> Result<Vec<Entity>> {
  log::debug!("list_entities for user_id: {:?}", user_id);

  let result: Vec<Entity> = sqlx::query_as(
      format!(
          "SELECT entity_data.entity_id, entity_data.user_id, entity_file.file_id, entity_file.entity_type FROM entity_main \
          JOIN entity_data ON entity_data.combo_id = entity_main.combo_id \
          JOIN entity_file ON entity_file.entity_id = entity_data.entity_id \
          WHERE entity_data.user_id = $1 \
          GROUP BY entity_main.combo_id \
          ORDER BY {} \
          LIMIT 50 OFFSET $2",
          sort.to_sql()
      ).as_str()
  )
      .bind(user_id)
      .bind(page * 50)
      .fetch_all(db)
      .await?;

  log::debug!("list_entities result: {:?}", result);

  Ok(result)
}
