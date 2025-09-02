use crate::config::Config;
use crate::sources::TarySource;
use crate::storage::Storage;
use log::trace;
use std::sync::Arc;
use teloxide::prelude::*;

const TELEGRAM_TOKEN: &str = "TELOXIDE_TOKEN";

pub struct Telegram {
    bot: Bot,
}

impl TarySource for Telegram {
    async fn listen(self) {
        trace!("Starting Telegram listener");

        teloxide::repl(self.bot.clone(), |bot: Bot, msg: Message| async move {
            bot.send_dice(msg.chat.id).await?;
            Ok(())
        })
        .await;
    }

    fn init(cfg: Arc<Config>, storage: Arc<Storage>) -> Option<Box<Self>> {
        if let Some(t) = &cfg.sources.as_ref().unwrap().telegram {
            if !t.enabled {
                return None;
            }
            info!("Setting up Telegram");

            // teloxide requires loading token from environment.
            trace!("Loading telegram token");
            let token = storage
                .get_secret(TELEGRAM_TOKEN)
                .expect("Telegram token not in secrets database!");

            unsafe {
                std::env::set_var(TELEGRAM_TOKEN, &token);
            }
            assert_eq!(std::env::var(TELEGRAM_TOKEN), Ok(token.to_string()));

            Some(Box::new(Self {
                bot: Bot::from_env(),
            }))
        } else {
            None
        }
    }
}

impl Telegram {}
