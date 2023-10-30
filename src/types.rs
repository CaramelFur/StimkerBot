use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use teloxide::{
    adaptors::DefaultParseMode, dispatching::dialogue::InMemStorage, prelude::Dialogue,
    types::FileMeta, Bot,
};

use crate::database::entities::EntityType;

pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
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
    }
}

#[derive(Clone, PartialEq)]
pub enum EntitySort {
    LastAdded,
    FirstAdded,
    LastUsed,
    FirstUsed,
    MostUsed,
    LeastUsed,
}

