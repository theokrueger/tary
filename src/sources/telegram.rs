use crate::config::Config;
use crate::content::Content;
use crate::sources::TarySource;
use crate::storage::Storage;
use chrono::prelude::*;
use log::{trace, warn};
use std::sync::Arc;
use teloxide::types::{ChatId, MediaKind, MessageKind, UserId};
use teloxide::{prelude::*, utils::command::BotCommands};

const TELEGRAM_TOKEN: &str = "TELOXIDE_TOKEN";

/// Supported Telegram source commands:
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    /// Display this text.
    #[command(aliases = ["h", "?"])]
    Help,
    /// Create a new TODO entry
    #[command(alias = "t")]
    Todo(String),
    ///// Create a new TODO entry with a due date
    //#[command(parse_with = "split", alias = "td", hide_aliases)]
    //TodoDate(String),
    ///// Create a new TODO entry with a due date and specific time
    //#[command(parse_with = "split", alias = "tdt", hide_aliases)]
    //TodoDateTime(String),
}

pub struct Telegram {
    bot: Bot,
    allowed_user: Option<UserId>,
}

impl TarySource for Telegram {
    async fn listen(self) {
        trace!("Starting Telegram listener");

        teloxide::repl(self.bot.clone(), move |bot: Bot, msg: Message| async move {
            if let Some(ref from) = msg.from {
                if self.allowed_user.is_none() || self.allowed_user.unwrap() == from.id {
                    if let MessageKind::Common(ref mc) = msg.kind
                        && let MediaKind::Text(mt) = &mc.media_kind
                    {
                        if let Ok(cmd) = Command::parse(mt.text.as_str(), "") {
                            handler(bot, msg, cmd).await;
                        }
                    }
                } else {
                    warn!(
                        "Received message from {}, who is not on the whitelist!",
                        from.id
                    );
                }
            }
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
                allowed_user: t.user,
            }))
        } else {
            None
        }
    }
}

impl Telegram {}

async fn handler(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    info!("Telegram message received");
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Todo(s) => {
            info!("{s}");
        }
    }

    Ok(())
}
