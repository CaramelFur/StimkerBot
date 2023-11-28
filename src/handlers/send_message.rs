use teloxide::{
    payloads::{SendMessage, SendMessageSetters},
    requests::{JsonRequest, Requester},
    types::{KeyboardButton, KeyboardMarkup, KeyboardRemove, Recipient, ReplyMarkup},
};

use crate::types::BotType;

pub trait BetterSendMessage {
    fn send_message_buttons<C, T, S>(
        &self,
        chat_id: C,
        text: T,
        buttons: Vec<S>,
    ) -> JsonRequest<SendMessage>
    where
        C: Into<Recipient>,
        T: Into<String>,
        S: Into<String>;

    fn send_message_easy<C, T>(&self, chat_id: C, text: T) -> JsonRequest<SendMessage>
    where
        C: Into<Recipient>,
        T: Into<String>,
    {
        self.send_message_buttons(chat_id, text, vec![] as Vec<&str>)
    }
}

impl BetterSendMessage for BotType {
    fn send_message_buttons<C, T, S>(
        &self,
        chat_id: C,
        text: T,
        buttons: Vec<S>,
    ) -> JsonRequest<SendMessage>
    where
        C: Into<Recipient>,
        T: Into<String>,
        S: Into<String>,
    {
        let mut message = self.send_message(chat_id, text);

        if buttons.is_empty() {
            message = message.reply_markup(ReplyMarkup::KeyboardRemove(KeyboardRemove::new()));
        } else {
            let buttons = buttons
                .into_iter()
                .map(|b| KeyboardButton::new(b.into()))
                .collect::<Vec<KeyboardButton>>();

            message = message.reply_markup(ReplyMarkup::Keyboard(
                KeyboardMarkup::new(vec![buttons])
                    .resize_keyboard(true)
                    .one_time_keyboard(true),
            ));
        }

        message
    }
}
