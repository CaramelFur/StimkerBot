use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use teloxide::types::*;

use crate::types::HandlerResult;

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize, sqlx::Type)]
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
    pub created_at: i64,
}

impl Entity {
    pub fn file_id_to_type<T: Into<String>>(file_id: T) -> HandlerResult<EntityType> {
        let file_id: String = file_id.into();
        // decode base64
        let file_id_decoded = general_purpose::URL_SAFE_NO_PAD.decode(file_id.as_bytes())?;
        let rle_decoded = rle_decode(&file_id_decoded);
        let mut type_num: u32 = u32::from_le_bytes(rle_decoded[0..4].try_into()?);

        type_num &= !(1 << 24 | 1 << 25);

        match type_num {
            2 => Ok(EntityType::Photo),
            4 => Ok(EntityType::Video),
            8 => Ok(EntityType::Sticker),
            10 => Ok(EntityType::Animation),
            _ => Err("Unknown file type".into()),
        }
    }

    pub fn to_inline(&self) -> InlineQueryResult {
        match self.entity_type {
            EntityType::Sticker => InlineQueryResult::CachedSticker(self.to_cached_sticker()),
            EntityType::Animation => InlineQueryResult::CachedGif(self.to_cached_animation()),
            EntityType::Photo => InlineQueryResult::CachedPhoto(self.to_cached_photo()),
            EntityType::Video => InlineQueryResult::CachedVideo(self.to_cached_video()),
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

fn rle_decode(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::<u8>::new();
    let mut last_is_zero = false;

    for byte in input {
        if last_is_zero {
            for _ in 0..(*byte - 1) {
                output.push(0x00);
            }
            last_is_zero = false;
        } else {
            output.push(*byte);
            if *byte == 0x00 {
                last_is_zero = true;
            }
        }
    }

    output
}
