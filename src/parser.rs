use chrono::{DateTime, Utc};
use std::lazy::SyncLazy;
use twilight_command_parser::{Command, CommandParserConfig, Parser};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::payload::MessageCreate;
use webhook::Webhook;

use crate::{
    models::{characters::Character, webhooks::get_or_create_webhook},
    BOT_USER_ID,
};

// When adding a new command, please keep this in sync
// with `PARSER` and `parse_command`.
const HELP: &str = r#"Available commands:
    `~help` – Displays this help message.
    `~echo message` –  Echo a message.
    `~ping` – Check the bot's response time.
    `~ls` – List your characters.
    `~create name avatar prefix` – Create a new character.
Arguments can be surrounded by quotes to allow for spaces.
For example, `~create "Hata no Kokoro" https://example.com/kokoro.png koko:` will create a character named Hata no Kokoro.
"#;

// When adding a new command, please keep this in sync
// with `HELP` and `parse_command`.
static PARSER: SyncLazy<Parser<'static>> = SyncLazy::new(|| {
    let mut config = CommandParserConfig::new();
    config.add_prefix("~");
    config.add_command("help", true);
    config.add_command("echo", true);
    config.add_command("ping", true);
    config.add_command("ls", true);
    config.add_command("create", true);
    Parser::new(config)
});

/// Parses a message to check if it has a command.
///
/// If it does, this returns `Some(response)`; otherwise,
/// this returns `None`.
pub async fn parse_command(message: &MessageCreate) -> Option<String> {
    // Don't reply to our own messages!
    if message.author.id.0 == *BOT_USER_ID {
        return None;
    }

    match PARSER.parse(&message.content) {
        Some(Command {
            name, arguments, ..
        }) => {
            // When adding a new command, please keep this in sync
            // with `HELP` and `PARSER`.
            let res = match name {
                "help" => HELP.to_owned(),

                "echo" => {
                    if arguments.as_str().trim().is_empty() {
                        return Some("`~echo` takes 1 argument (`message`).".to_owned());
                    }

                    arguments.as_str().to_owned()
                },

                "ping" => {
                    if !arguments.as_str().trim().is_empty() {
                        return Some("`~ping` takes no arguments.".to_owned());
                    }

                    let message_timestamp =
                        DateTime::parse_from_rfc3339(&message.timestamp).unwrap();

                    let time = Utc::now()
                        .signed_duration_since(message_timestamp)
                        .num_milliseconds();

                    format!("Pong. Command received in **{}** ms.", time)
                },

                "ls" => {
                    if !arguments.as_str().trim().is_empty() {
                        return Some("`~ls` takes no arguments.".to_owned());
                    }

                    let characters = Character::all_with_user_id(message.author.id.0)
                        .await
                        .unwrap();

                    if characters.is_empty() {
                        "You have no characters.".to_owned()
                    } else {
                        characters
                            .iter()
                            .map(|ch| format!("{} (prefix `{}`)", ch.char_name, ch.char_prefix))
                            .intersperse("\n".to_owned())
                            .collect()
                    }
                },

                "create" => {
                    let arguments_vec = arguments.clone().collect::<Vec<_>>();

                    if arguments_vec.len() != 3 {
                        return Some(
                            "`~create` takes 3 arguments (`name`, `avatar`, `prefix`).".to_owned(),
                        );
                    }

                    let name = arguments_vec[0];
                    let avatar = arguments_vec[1];
                    let prefix = arguments_vec[2];

                    Character::insert(message.author.id.0, name, avatar, prefix)
                        .await
                        .unwrap();

                    format!("Created {} successfully.", name)
                },

                // `PARSER` will only return `Some` for commands
                // that have been explicitly added. Please keep
                // this match expression in sync with `HELP` and
                // `PARSER`.
                _ => unreachable!(),
            };

            Some(res)
        },

        _ => None,
    }
}

/// Parse a message, checking to see if it contains a prefix that matches
/// one of the message author's characters.
pub async fn parse_prefix(http: &HttpClient, message: &MessageCreate) {
    let characters = Character::all_with_user_id(message.author.id.0)
        .await
        .unwrap();

    for character in characters {
        if let Some(message_no_prefix) = message.content.strip_prefix(&character.char_prefix) {
            let webhook = Webhook::from_url(
                &get_or_create_webhook(&http, message.channel_id.0)
                    .await
                    .unwrap(),
            );

            webhook
                .send(|res| {
                    res.username(&character.char_name)
                        .avatar_url(&character.char_avatar)
                        .content(message_no_prefix.trim())
                })
                .await
                .unwrap();

            http.delete_message(message.channel_id, message.id)
                .await
                .unwrap();

            // We're done
            return;
        }
    }
}
