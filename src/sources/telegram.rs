use crate::config::Config;
use crate::content::Content;
use crate::sources::TarySource;
use crate::storage::Storage;
use chrono::prelude::*;
use dptree::prelude::*;
use log::{trace, warn};
use std::error::Error;
use std::sync::Arc;
use teloxide::types::{ChatId, MediaKind, MessageKind, User, UserId};
use teloxide::{prelude::*, utils::command::BotCommands};
use tokio::sync::broadcast::Sender;

type HandlerResult = Result<(), Box<dyn Error + Send + Sync>>;
const TELEGRAM_TOKEN: &str = "TELOXIDE_TOKEN";

/// Supported Telegram source commands:
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    /// Display this text.
    #[command(aliases = ["h", "?", "start"])]
    Help,
    /// Create a new TODO entry
    /// Specify due date with [DUE:YYYY-MM-DD HH:MM]
    /// Specify destinatioh with [DEST:Location]
    #[command(alias = "t")]
    Todo(String),
}

pub struct Telegram {
    bot: Bot,
    allowed_user: Option<UserId>,
}

impl TarySource for Telegram {
    async fn listen(self, tx: Sender<Content>) {
        trace!("Starting Telegram listener");

        let schema = Update::filter_message()
            .filter_map(|update: Update| update.from().cloned())
            .filter(move |user: User| {
                self.allowed_user.is_none() || self.allowed_user.unwrap() == user.id
            })
            .chain(Message::filter_text())
            .endpoint(Self::handler);

        Dispatcher::builder(self.bot, schema)
            .dependencies(dptree::deps![tx])
            //.enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    }

    fn init(cfg: Arc<Config>, storage: Arc<Storage>) -> Option<Box<Self>> {
        if let Some(t) = &cfg.sources.telegram
            && t.enabled
        {
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

impl Telegram {
    async fn handler(bot: Bot, tx: Sender<Content>, user: User, text: String) -> HandlerResult {
        let cmd = match Command::parse(text.as_str(), "") {
            Ok(c) => c,
            Err(e) => {
                bot.send_message(user.id, "Invalid command formatting!")
                    .await?;
                return Err(e.into());
            }
        };

        info!("Telegram message received");

        let username: String = if let Some(n) = user.username {
            format!("Telegram - @{n}")
        } else {
            let l = user.last_name.unwrap_or("".to_string());
            format!(
                "Telegram - {f}{space}{l}",
                f = user.first_name,
                space = if !l.is_empty() { "" } else { " " }
            )
        };

        match cmd {
            Command::Help => {
                bot.send_message(user.id, Command::descriptions().to_string())
                    .await?;
            }
            Command::Todo(mut s) => {
                if s.is_empty() {
                    return Ok(());
                }
                let mut c = Content::new(username, None);
                // search for due
                if let Some(i) = s.find("[DUE:")
                    && let Some(j) = s[i..].find("]")
                {
                    let k = i + j;
                    let d = &s[i + 5..k];
                    if let Ok(ndt) = NaiveDateTime::parse_from_str(d, "%Y-%m-%d %H:%M") {
                        c.due = Some(Local.from_local_datetime(&ndt).unwrap());
                    } else {
                        bot.send_message(user.id, "Invalid due date formatting!")
                            .await?;
                    }
                    s = "".to_string() + &s[..i] + &s[k + 1..];
                }
                // search for destination
                if let Some(i) = s.find("[DEST:")
                    && let Some(j) = s[i..].find("]")
                {
                    let k = i + j;
                    let d = &s[i + 5..k];
                    c.dest = Some(d.to_string());
                    s = "".to_string() + &s[..i] + &s[k + 1..];
                }

                c.content = s;
                trace!("Telegram source sending TODO");
                tx.send(c).unwrap();
            }
        };

        Ok(())
    }
}
