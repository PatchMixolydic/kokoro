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

macro_rules! parser {
    (
        with ($message:ident, $arguments:ident) {
            $($command:ident $(($help:literal))? => $exp:expr $(,)?)*
        }
    ) => {
        static PARSER: SyncLazy<Parser<'static>> = SyncLazy::new(|| {
            let mut config = CommandParserConfig::new();
            config.add_prefix("~");
            config.add_command("help", true);
            $(config.add_command(stringify!($command), true);)*

            Parser::new(config)
        });

        pub async fn parse_command($message: &MessageCreate) -> Option<String> {
            if $message.author.id.0 == *BOT_USER_ID {
                return None;
            }

            match PARSER.parse(&$message.content) {
                Some(Command {
                    name, $arguments, ..
                }) => Some(match name {
                    "help" => {
                        concat!(
                            $(
                                "~",
                                stringify!($command),
                                $(" - ", $help,)?
                                "\n",
                            )*
                            "Arguments can be surrounded by quotes to allow for spaces.\n",
                            "For example, `~create \"Hata no Kokoro\" https://example.com/kokoro.png koko:` ",
                            "will create a character named \"Hata no Kokoro\"",
                        ).to_owned()
                    },

                    $(stringify!($command) => $exp,)*
                    _ => unreachable!(),
                }),

                _ => None,
            }
        }
    };
}

parser! {
    with (message, arguments) {
        echo("Respond with the given message") => arguments.as_str().to_owned(),

        ping("Checks to see if the bot is connected") => {
            let message_timestamp = DateTime::parse_from_rfc3339(&message.timestamp).unwrap();
            let time = Utc::now().signed_duration_since(message_timestamp).num_milliseconds();
            format!("Pong. Command recieved in **{}** ms.", time)
        }

        ls("Lists all of your characters") => {
            let characters = Character::all_with_user_id(message.author.id.0).await.unwrap();

            if characters.is_empty() {
                "You have no characters.".to_owned()
            } else {
                characters
                    .iter()
                    .map(|ch| format!("{} (prefix `{}`)", ch.char_name, ch.char_prefix))
                    .intersperse("\n".to_owned())
                    .collect()
            }
        }

        create("Create a new character") => {
            let arguments = arguments.collect::<Vec<_>>();

            if arguments.len() != 3 {
                return Some("\
                    ~create takes 3 arguments: `name`, `avatar`, and `prefix`.\n\
                    For example: `~create Kokoro http://example.com/kokoro.png koko:`.
                ".to_owned());
            }

            Character::insert(message.author.id.0, arguments[0], arguments[1], arguments[2]).await.unwrap();
            format!("Created {} successfully.", arguments[0])
        }
    }
}

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
