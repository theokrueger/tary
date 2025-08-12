use crate::config::Config;
use crate::sources::SourceListener;
use crate::storage::Storage;
use log::trace;
use std::sync::Arc;
use teloxide::prelude::*;

const TELEGRAM_TOKEN: &str = "TELOXIDE_TOKEN";

pub struct Telegram {
    bot: Bot,
}

impl SourceListener for Telegram {
    async fn listen(&self) {
        teloxide::repl(self.bot.clone(), |bot: Bot, msg: Message| async move {
            bot.send_dice(msg.chat.id).await?;
            Ok(())
        })
        .await;
    }

    fn init(cfg: Arc<Config>, storage: Arc<Storage>) -> Self {
        trace!("Loading telegram token");

        // teloxide requires loading token from environment.
        let token = storage
            .get_secret(TELEGRAM_TOKEN)
            .expect("Telegram token not in secrets database!");

        unsafe {
            std::env::set_var(TELEGRAM_TOKEN, &token);
        }
        assert_eq!(std::env::var(TELEGRAM_TOKEN), Ok(token.to_string()));

        Self {
            bot: Bot::from_env(),
        }
    }
}

impl Telegram {}
