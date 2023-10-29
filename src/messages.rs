use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{FileMeta, Me};
use teloxide::utils::command::BotCommands;

use crate::database::entities::EntityType;
use crate::database::queries::{self, InsertEntity};
use crate::types::*;
use crate::util::unix_to_humantime;

#[derive(BotCommands)]
#[command(rename_rule = "lowercase")]
enum Command {
    #[command(description = "Show this text")]
    Help,

    #[command(description = "Show info about this bot")]
    Start,

    #[command(description = "Add or remove tags to an entire stickerpack")]
    Pack,

    #[command(description = "Export your data to a json")]
    Export,

    #[command(description = "Import your data from a json")]
    Import,

    #[command(description = "Stop whatever you are doing")]
    Cancel,

    #[command(description = "DANGEROUS! Wipes your data")]
    Stop,
}

pub async fn command_handler(
    bot: BotType,
    me: Me,
    dialogue: DialogueWithState,
    msg: Message,
) -> HandlerResult {
    match Command::parse(msg.text().unwrap(), me.username()) {
        Ok(Command::Help) => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Ok(Command::Start) => {
            bot.send_message(
                msg.chat.id,
                "You can start using this bot by sending it a sticker, gif, photo or video",
            )
            .await?;
        }
        Ok(Command::Pack) => {
            if dialogue.get().await?.unwrap() != ConversationState::ReceiveEntityId {
                bot.send_message(
                    msg.chat.id,
                    "Please finish your action, or send /cancel to cancel",
                )
                .await?;
                return Ok(());
            }

            bot.send_message(
                msg.chat.id,
                "Please send me a sticker from the pack you want to tag",
            )
            .await?;

            dialogue
                .update(ConversationState::RecieveEntitiesId)
                .await?;
        }
        Ok(Command::Export) => {
            bot.send_message(msg.chat.id, "TODO").await?;
        }
        Ok(Command::Import) => {
            bot.send_message(msg.chat.id, "TODO").await?;
        }
        Ok(Command::Cancel) => {
            dialogue.update(ConversationState::ReceiveEntityId).await?;
            bot.send_message(msg.chat.id, "Cancelled").await?;
        }
        Ok(Command::Stop) => {
            bot.send_message(
                msg.chat.id,
                "Please send 'I WANT TO DELETE EVERYTHING' to confirm",
            )
            .await?;
            dialogue.update(ConversationState::VerifyStop).await?;
        }
        Err(_) => {
            bot.send_message(msg.chat.id, "Unknown command").await?;
        }
    }

    return Ok(());
}

pub async fn verify_stop(
    db: Arc<DbConn>,
    bot: BotType,
    dialogue: DialogueWithState,
    msg: Message,
) -> HandlerResult {
    dialogue.update(ConversationState::ReceiveEntityId).await?;

    if msg.text().is_none() || msg.text().unwrap() != "I WANT TO DELETE EVERYTHING" {
        bot.send_message(msg.chat.id, "Stop action cancelled")
            .await?;
        return Ok(());
    }

    let user_id = msg.from().unwrap().id.to_string();

    log::debug!("Wiping user {:?}", user_id);

    queries::wipe_user(&db, user_id.clone()).await?;

    bot.send_message(msg.chat.id, "All your data has been wiped")
        .await?;

    Ok(())
}

pub async fn receive_entities_ids(
    _db: Arc<DbConn>,
    bot: BotType,
    dialogue: DialogueWithState,
    msg: Message,
) -> HandlerResult {
    if msg.sticker().is_none() {
        bot.send_message(
            msg.chat.id,
            "Please send me a sticker from the pack you want to tag",
        )
        .await?;
        return Ok(());
    }

    let sticker = msg.sticker().unwrap();
    if sticker.set_name.is_none() {
        bot.send_message(
            msg.chat.id,
            "This sticker doesn't belong to a pack, please send me a sticker from a pack",
        )
        .await?;
        return Ok(());
    }

    let pack_name = sticker.set_name.as_ref().unwrap();
    log::debug!("Got pack name: {:?}", pack_name);

    let pack = bot.get_sticker_set(pack_name).await?;

    let entities: Vec<FileMeta> = pack
        .stickers
        .iter()
        .map(|sticker| sticker.file.to_owned())
        .collect();

    bot.send_message(
        msg.chat.id,
        format!(
            "Got stickerpack <code>{}</code> with <code>{}</code> stickers.\n\
            Which tags do you want to add to this?\n\
            - Start the tag with <code>-</code> to remove an existing tag",
            pack_name,
            entities.len()
        ),
    )
    .await?;

    dialogue
        .update(ConversationState::RecieveEntitiesTags { entities })
        .await?;

    Ok(())
}

pub async fn receive_entities_tags(
    db: Arc<DbConn>,
    bot: BotType,
    dialogue: DialogueWithState,
    msg: Message,
    entities: Vec<FileMeta>,
) -> HandlerResult {
    let entity_type = EntityType::Sticker;

    if msg.text().is_none() {
        bot.send_message(
            msg.chat.id,
            "Please send me a space seperated list of tags or /cancel",
        )
        .await?;
        return Ok(());
    }

    let user_id = msg.from().unwrap().id.to_string();
    let tags: Vec<String> = msg
        .text()
        .unwrap()
        .to_lowercase()
        .replace(",", " ")
        .split(" ")
        .map(|s| s.trim().to_string())
        .filter(|s| s.len() > 0)
        .collect();

    log::debug!("Got tags: {:?} from {:?}", tags, user_id);

    if tags.len() == 0 {
        bot.send_message(msg.chat.id, "No tags provided").await?;
        return Ok(());
    }

    // split the tags into add and remove
    let remove_tags = tags
        .iter()
        .filter(|tag| tag.starts_with("-"))
        .map(|tag| tag.replace("-", ""))
        .collect::<Vec<String>>();

    let add_tags = tags
        .iter()
        .filter(|tag| !tag.starts_with("-"))
        .map(|tag| tag.to_string())
        .collect::<Vec<String>>();

    log::debug!("Removing tags: {:?}", remove_tags);
    log::debug!("Adding tags: {:?}", add_tags);

    let processing_message = bot
        .send_message(
            msg.chat.id,
            format!("Processing <code>{}</code> stickers...", entities.len()),
        )
        .await?;

    let insert_entities: Vec<InsertEntity> = entities
        .iter()
        .map(|entity| InsertEntity {
            entity_id: entity.unique_id.clone(),
            file_id: entity.id.clone(),
        })
        .collect();

    queries::insert_tags(
        &db,
        user_id.clone(),
        insert_entities,
        entity_type.clone(),
        add_tags.clone(),
    )
    .await?;

    queries::remove_tags(
        &db,
        user_id.clone(),
        entities.iter().map(|e| e.unique_id.clone()).collect(),
        remove_tags.clone(),
    )
    .await?;

    bot.edit_message_text(
        processing_message.chat.id,
        processing_message.id,
        format!(
            "Success!\n\
            Added tags: <b>{}</b>\n\
            Removed tags: <b>{}</b>",
            add_tags.join(", "),
            remove_tags.join(", ")
        ),
    )
    .await?;
    dialogue.update(ConversationState::ReceiveEntityId).await?;
    Ok(())
}

pub async fn receive_entity_id(
    db: Arc<DbConn>,
    bot: BotType,
    dialogue: DialogueWithState,
    msg: Message,
) -> HandlerResult {
    let (entity, entity_type) = if msg.sticker().is_some() {
        (msg.sticker().unwrap().file.to_owned(), EntityType::Sticker)
    } else if msg.animation().is_some() {
        (
            msg.animation().unwrap().file.to_owned(),
            EntityType::Animation,
        )
    } else if msg.video().is_some() {
        (msg.video().unwrap().file.to_owned(), EntityType::Video)
    } else if msg.photo().is_some() {
        (
            msg.photo().unwrap().first().unwrap().file.to_owned(),
            EntityType::Photo,
        )
    } else {
        bot.send_message(
            msg.chat.id,
            "Please send me a sticker, animation, photo or video",
        )
        .await?;
        return Ok(());
    };

    let user_id = msg.from().unwrap().id.to_string();

    log::debug!(
        "Got entity: {:?} file: {:?} from: {:?}",
        entity.unique_id,
        entity.id,
        user_id
    );

    let mut current_tags =
        queries::get_tags(&db, user_id.clone(), entity.unique_id.clone()).await?;
    current_tags.sort();

    log::debug!("Current tags: {:?}", current_tags);

    let entity_usage =
        queries::get_entity_usage(&db, user_id.clone(), entity.unique_id.clone()).await?;

    if current_tags.len() > 0 {
        bot.send_message(
            msg.chat.id,
            format!(
                "Your current tags for this are: <b>{}</b>\n\
                You've used this <code>{}</code> times\n\
                You've last used this <code>{}</code>",
                current_tags.join(", "),
                entity_usage.count,
                unix_to_humantime(entity_usage.last_used)
            ),
        )
        .await?;
    }

    bot.send_message(
        msg.chat.id,
        "Which tags do you want to add to this?\n\
        - Make the first tag <code>replace</code>, to replace all tags\n\
        - Make the first tag <code>clear</code>, to remove all existing tags\n\
        - Start the tag with <code>-</code> to remove an existing tag",
    )
    .await?;

    dialogue
        .update(ConversationState::ReceiveEntityTags {
            entity,
            entity_type,
        })
        .await?;

    Ok(())
}

pub async fn receive_entity_tags(
    db: Arc<DbConn>,
    bot: BotType,
    dialogue: DialogueWithState,
    msg: Message,
    entity: FileMeta,
    entity_type: EntityType,
) -> HandlerResult {
    if msg.text().is_none() {
        bot.send_message(
            msg.chat.id,
            "Please send me a space seperated list of tags or /cancel",
        )
        .await?;
        return Ok(());
    }

    let user_id = msg.from().unwrap().id.to_string();
    let mut tags: Vec<String> = msg
        .text()
        .unwrap()
        .to_lowercase()
        .replace(",", " ")
        .split(" ")
        .map(|s| s.trim().to_string())
        .filter(|s| s.len() > 0)
        .collect();

    log::debug!("Got tags: {:?} from {:?}", tags, user_id);

    if tags[0] == "replace" || tags[0] == "clear" {
        log::debug!("Wiping tags");
        queries::wipe_tags(&db, user_id.clone(), entity.unique_id.clone()).await?;

        if tags[0] == "clear" {
            bot.send_message(msg.chat.id, "Cleared all tags for this")
                .await?;
            dialogue.update(ConversationState::ReceiveEntityId).await?;
            return Ok(());
        }

        tags.remove(0);
    }

    if tags.len() == 0 {
        bot.send_message(msg.chat.id, "No tags provided").await?;
        return Ok(());
    }

    // split the tags into add and remove
    let remove_tags = tags
        .iter()
        .filter(|tag| tag.starts_with("-"))
        .map(|tag| tag.replace("-", ""))
        .collect::<Vec<String>>();

    let add_tags = tags
        .iter()
        .filter(|tag| !tag.starts_with("-"))
        .map(|tag| tag.to_string())
        .collect::<Vec<String>>();

    log::debug!("Removing tags: {:?}", remove_tags);
    log::debug!("Adding tags: {:?}", add_tags);

    queries::insert_tags(
        &db,
        user_id.clone(),
        vec![InsertEntity {
            entity_id: entity.unique_id.clone(),
            file_id: entity.id.clone(),
        }],
        entity_type.clone(),
        add_tags,
    )
    .await?;

    queries::remove_tags(
        &db,
        user_id.clone(),
        vec![entity.unique_id.clone()],
        remove_tags,
    )
    .await?;

    tags = queries::get_tags(&db, user_id.clone(), entity.unique_id.clone()).await?;
    tags.sort();

    log::debug!("New tags for user {:?} is {:?}", user_id, tags);

    bot.send_message(
        msg.chat.id,
        format!("The new tags for this are now: <b>{}</b>", tags.join(", ")),
    )
    .await?;
    dialogue.update(ConversationState::ReceiveEntityId).await?;
    Ok(())
}
