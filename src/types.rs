use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use teloxide::{
    adaptors::DefaultParseMode, dispatching::dialogue::InMemStorage, prelude::Dialogue,
    types::FileMeta, Bot,
};

pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
pub type DialogueWithState = Dialogue<ConversationState, InMemStorage<ConversationState>>;

pub type BotType = DefaultParseMode<Bot>;
pub type DbConn = Pool<Sqlite>;
pub type DbType = Arc<DbConn>;

#[derive(Clone, Default)]
pub enum ConversationState {
    #[default]
    ReceiveEntityID,
    ReceiveEntityTags {
        entity: FileMeta,
    },
}
