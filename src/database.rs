use entity::sticker_tag;
use entity::tag;

use migration::OnConflict;
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use sea_orm::DatabaseConnection;

pub async fn insert_tag(
    db: &DatabaseConnection,
    user_id: String,
    sticker_id: String,
    tag_name: String,
) -> Result<(), DbErr> {
    log::debug!("insert_tag: {:?} for sticker_id: {:?} and user_id: {:?}", tag_name, sticker_id, user_id);

    // Try to find existing tag ID
    let existing_tag_id = tag::Entity::find()
        .filter(tag::Column::Tag.eq(tag_name.clone()))
        .one(db)
        .await?
        .map(|tag| tag.tag_id);

    log::debug!("existing_tag_id: {:?}", existing_tag_id);

    let tag_id = match existing_tag_id {
        Some(tag_id) => tag_id,
        None => {
            tag::Entity::insert(tag::ActiveModel {
                tag: Set(tag_name.clone()),
                ..Default::default()
            })
            .exec(db)
            .await?
            .last_insert_id
        }
    };

    log::debug!("tag_id: {:?}", tag_id);

    let new_tag_association: sticker_tag::ActiveModel = sticker_tag::Model {
        sticker_id,
        user_id,
        tag_id,
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

// pub async fn find_stickers(
//     db: &DatabaseConnection,
//     user_id: String,
//     tags: Vec<String>,
// ) -> Result<Vec<String>, DbErr> {
//     let mut query = sticker_tag::Entity::find();

//     // return all sticker_ids where user_id is equal to user_id and the tag is equal to all tags

//     Ok(stickers
//         .into_iter()
//         .map(|sticker| sticker.sticker_id)
//         .collect())
// }
