use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use teloxide::{
    adaptors::DefaultParseMode, dispatching::dialogue::InMemStorage, prelude::Dialogue,
    types::FileMeta, Bot,
};

use crate::database::EntityType;

pub type HandlerResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub type DialogueWithState = Dialogue<ConversationState, InMemStorage<ConversationState>>;

pub type BotType = DefaultParseMode<Bot>;
pub type DbConn = Pool<Sqlite>;
pub type DbType = Arc<DbConn>;

#[derive(Clone, Default, PartialEq)]
pub enum ConversationState {
    #[default]
    ReceiveEntityId,
    ReceiveEntityTags {
        entity: FileMeta,
        entity_type: EntityType,
    },

    VerifyStop,

    RecieveEntitiesId,
    RecieveEntitiesTags {
        entities: Vec<FileMeta>,
    },

    ReceiveQSImport,
    ReceiveBotImport,
}

#[derive(Clone, PartialEq, Default)]
pub enum EntitySort {
    LastAdded,
    FirstAdded,
    LastUsed,
    FirstUsed,
    #[default]
    MostUsed,
    LeastUsed,
    Random,
}

impl EntitySort {
    pub fn to_sql(&self) -> &'static str {
        match self {
            EntitySort::LastAdded => "entity_data.created_at DESC",
            EntitySort::FirstAdded => "entity_data.created_at ASC",
            EntitySort::LastUsed => "entity_data.last_used DESC",
            EntitySort::FirstUsed => "entity_data.last_used ASC",
            EntitySort::MostUsed => "entity_data.count DESC",
            EntitySort::LeastUsed => "entity_data.count ASC",
            EntitySort::Random => "RANDOM()",
        }
    }
}

#[derive(Clone, PartialEq, Default)]
pub struct InlineSearchQuery {
    pub tags: Vec<String>,
    pub negative_tags: Vec<String>,
    pub sort: EntitySort,
    pub entity_type: Option<EntityType>,
    pub get_all: bool,
}
