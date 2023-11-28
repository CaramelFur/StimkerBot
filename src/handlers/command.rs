use std::sync::Arc;

use anyhow::Result;
use teloxide::{macros::BotCommands, types::{Me, Message}, utils::command::BotCommands as _};

use crate::{types::{DbConn, BotType, DialogueWithState, ConversationState}, database::queries};
use super::{send_message::BetterSendMessage, import::send_fix_entities};
use super::import::send_bot_export;

#[derive(BotCommands)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "Show all help for this bot")]
    Help,

    #[command(description = "Start using this bot")]
    Start,

    #[command(description = "Add or remove tags to an entire stickerpack")]
    Pack,

    #[command(description = "Stop whatever you are doing")]
    Cancel,

    #[command(description = "Export your data")]
    Export,

    #[command(description = "Import your data")]
    Import,

    #[command(description = "Import your data from a QuickStickBot or QuickGifBot export")]
    QSImport,

    #[command(description = "If stickerbot is not longer working, try this. (This is a slow operation, use sparingly)")]
    FixEntities,

    #[command(description = "Shows global statistics about this bot")]
    Stats,

    #[command(description = "Shows information about this bot")]
    About,

    #[command(description = "DANGEROUS! Wipes your data")]
    Stop,
}

pub async fn receive_command(
    db: Arc<DbConn>,
    bot: BotType,
    me: Me,
    dialogue: DialogueWithState,
    msg: Message,
) -> Result<()> {
    match Command::parse(msg.text().unwrap(), me.username()) {
        Ok(Command::Help) => {
            let command_help = Command::descriptions().to_string();
            bot.send_message_easy(
                msg.chat.id,
                format!(
                    "<b>General</b>\n\
                    This bot allows you to tag your stickers, gifs, photos and videos with tags. \
                    You can then search for these tags and send the sticker, gif, photo or video. \
                    Start a search by mentioning me in your chatbox.\n\
                    \n<b>Tagging</b>\n\
                    You can tag your stickers, gifs, photos and videos by sending them to me. \
                    I will then ask you which tags you want to add to it. \
                    If you want to tag an enitre stickerpack, use the /pack command.\n\
                    \n<b>Filters</b>\n\
                    You can filter your search by typing in a tag, or multiple tags. \
                    Tags are searched for with autocomplete, so you don't have to type the full tag. \
                    You can negate tags  by appending <code>-</code> to a tag. \
                    You can use some special filters to narrow down your search.\n\
                    - <code>all</code> will show all your stickers, gifs, photos and videos\n\
                    - <code>sticker</code> or <code>stk</code> will only show stickers\n\
                    - <code>animation</code> or <code>gif</code> will only show gifs\n\
                    - <code>photo</code> or <code>pic</code> will only show photos\n\
                    - <code>video</code> or <code>vid</code> will only show videos\n\
                    \n<b>Sorting</b>\n\
                    You can sort your results by using the following filters:\n\
                    - <code>most_used</code> or <code>mu</code> will sort by most used\n\
                    - <code>least_used</code> or <code>lu</code> will sort by least used\n\
                    - <code>last_added</code> or <code>la</code> will sort by last added\n\
                    - <code>first_added</code> or <code>fa</code> will sort by first added\n\
                    - <code>last_used</code> or <code>nu</code> will sort by last used\n\
                    - <code>first_used</code> or <code>ou</code> will sort by first used\n\
                    - <code>random</code> or <code>rnd</code> will sort randomly\n\
                    \n<b>Commands</b>\n\
                    {}",
                    command_help
                ),
            )
            .await?;
        }
        Ok(Command::Start) => {
            bot.send_message_easy(
                msg.chat.id,
                "You can start using this bot by sending it a sticker, gif, photo or video.\n\
                You can also use /help to get more information",
            )
            .await?;
        }
        Ok(Command::Pack) => {
            if dialogue.get().await?.unwrap() != ConversationState::ReceiveEntityId {
                bot.send_message_easy(msg.chat.id, "Please finish your action, or /cancel")
                    .await?;
                return Ok(());
            }

            bot.send_message_easy(
                msg.chat.id,
                "Please send me a sticker from the pack you want to tag",
            )
            .await?;

            dialogue
                .update(ConversationState::RecieveEntitiesId)
                .await?;
        }
        Ok(Command::Export) => {
            send_bot_export(&db, &bot, &msg).await?;
        }
        Ok(Command::Import) => {
            bot.send_message_buttons(
                msg.chat.id,
                "Ready to import, please send me the file you got from /export",
                vec!["/cancel"],
            )
            .await?;
            dialogue.update(ConversationState::ReceiveBotImport).await?;
        }
        Ok(Command::QSImport) => {
            bot.send_message_buttons(
                msg.chat.id,
                "Ready to import, please send me the file you got from QuickStickBot",
                vec!["/cancel"],
            )
            .await?;

            dialogue.update(ConversationState::ReceiveQSImport).await?;
        }
        Ok(Command::FixEntities) => {
            send_fix_entities(&db, &bot, &msg).await?;
        }
        Ok(Command::Cancel) => {
            dialogue.update(ConversationState::ReceiveEntityId).await?;
            bot.send_message_easy(msg.chat.id, "Cancelled").await?;
        }
        Ok(Command::Stats) => {
            let stats = queries::get_global_stats(&db).await?;
            bot.send_message_easy(
                msg.chat.id,
                format!(
                    "<b>Global stats</b>\n\
                    Users: <code>{}</code>\n\
                    Tags: <code>{}</code>\n\
                    Sent: <code>{}</code>\n\
                    \n\
                    <b>Entity count</b>\n\
                    <i>Total</i>: <code>{}</code>\n\
                    Stickers: <code>{}</code>\n\
                    Animations: <code>{}</code>\n\
                    Photos: <code>{}</code>\n\
                    Videos: <code>{}</code>",
                    stats.total_users,
                    stats.total_tags,
                    stats.total_entities_sent,
                    stats.total_stickers
                        + stats.total_animations
                        + stats.total_photos
                        + stats.total_videos,
                    stats.total_stickers,
                    stats.total_animations,
                    stats.total_photos,
                    stats.total_videos,
                ),
            )
            .await?;
        }
        Ok(Command::About) => {
            bot.send_message_easy(
                msg.chat.id,
                format!(
                    "<b>Stimkerbot V{}</b>\n\
                    This bot is made by @CaramelFluff (<a href=\"https://caramelfur.dev/\">caramelfur.dev</a>)\n\
                    The source code is available on <a href=\"https://github.com/CaramelFur/StimkerBot\">GitHub</a>",
                    env!("CARGO_PKG_VERSION")
                ),
            )
            .await?;
        }
        Ok(Command::Stop) => {
            bot.send_message_buttons(
                msg.chat.id,
                "Please send 'I WANT TO DELETE EVERYTHING' to confirm",
                vec!["/cancel"],
            )
            .await?;
            dialogue.update(ConversationState::VerifyStop).await?;
        }
        Err(_) => {
            bot.send_message_easy(msg.chat.id, "Unknown command")
                .await?;
        }
    }

    return Ok(());
}