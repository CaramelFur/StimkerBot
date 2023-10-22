use teloxide::{prelude::Dialogue, types::FileMeta, dispatching::dialogue::InMemStorage};

pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
pub type MyDialogue = Dialogue<ConversationState, InMemStorage<ConversationState>>;

#[derive(Clone, Default)]
pub enum ConversationState {
    #[default]
    ReceiveStickerID,
    ReceiveStickerTags {
        sticker: FileMeta,
    },
}
