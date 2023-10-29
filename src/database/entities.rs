use std::default;

use teloxide::types::*;

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum EntityType {
    Sticker,
    Animation,
    Photo,
    Video,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Entity {
    pub entity_id: String,
    pub file_id: String,
    pub user_id: String,
    pub entity_type: EntityType,
}

#[derive(Debug, sqlx::FromRow)]
pub struct EntityStat {
    pub user_id: String,
    pub entity_id: String,
    pub count: i64,
    pub last_used: i64,
}

impl Entity {
    pub fn to_inline(&self) -> InlineQueryResult {
        match self.entity_type {
            EntityType::Sticker => InlineQueryResult::CachedSticker(
                self.to_cached_sticker()
            ),
            EntityType::Animation => InlineQueryResult::CachedGif(
                self.to_cached_animation()
            ),
            EntityType::Photo => InlineQueryResult::CachedPhoto(
                self.to_cached_photo()
            ),
            EntityType::Video => InlineQueryResult::CachedVideo(
                self.to_cached_video()
            ),
        }
    }

    fn to_cached_sticker(&self) -> InlineQueryResultCachedSticker {
        InlineQueryResultCachedSticker {
            id: format!("{}", self.entity_id.to_owned()),
            sticker_file_id: self.file_id.to_owned(),
            input_message_content: None,
            reply_markup: None,
        }
    }

    fn to_cached_animation(&self) -> InlineQueryResultCachedGif {
        InlineQueryResultCachedGif {
            id: format!("{}", self.entity_id.to_owned()),
            gif_file_id: self.file_id.to_owned(),
            title: None,
            caption: None,
            input_message_content: None,
            reply_markup: None,
            parse_mode: None,
            caption_entities: None,
        }
    }

    fn to_cached_photo(&self) -> InlineQueryResultCachedPhoto {
        InlineQueryResultCachedPhoto {
            id: format!("{}", self.entity_id.to_owned()),
            photo_file_id: self.file_id.to_owned(),
            title: None,
            description: None,
            caption: None,
            parse_mode: None,
            reply_markup: None,
            input_message_content: None,
            caption_entities: None,
        }
    }

    fn to_cached_video(&self) -> InlineQueryResultCachedVideo {
        InlineQueryResultCachedVideo {
            id: format!("{}", self.entity_id.to_owned()),
            video_file_id: self.file_id.to_owned(),
            title: "Video".into(),
            description: None,
            caption: None,
            parse_mode: None,
            reply_markup: None,
            input_message_content: None,
            caption_entities: None,
        }
    }
}
