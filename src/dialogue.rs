use teloxide::{prelude::Dialogue, types::FileMeta, dispatching::dialogue::InMemStorage, adaptors::DefaultParseMode, Bot};

pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
pub type MyDialogue = Dialogue<ConversationState, InMemStorage<ConversationState>>;

pub type BotType = DefaultParseMode<Bot>;

#[derive(Clone, Default)]
pub enum ConversationState {
    #[default]
    ReceiveStickerID,
    ReceiveStickerTags {
        sticker: FileMeta,
    },
}
