#[macro_export]
macro_rules! count_idents {
    (@unit $id:ident) => {()};

    ($($id:ident)*) => {
        <[()]>::len(&[$($crate::count_idents!(@unit $id)),*])
    };
}

#[macro_export]
macro_rules! parser {
    (
        with ($message:ident, $arguments:ident) {
            $($command:ident ($($arg_name:ident),*) => $exp:expr $(,)?)*
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
                            "Available commands:\n",
                            $(
                                "    ~",
                                stringify!($command),
                                $(" ", stringify!($arg_name),)*
                                "\n",
                            )*
                            "\n",
                            "Arguments can be surrounded by quotes to allow for spaces.\n",
                            "For example, `~create \"Hata no Kokoro\" https://example.com/kokoro.png koko:` ",
                            "will create a character named \"Hata no Kokoro\"",
                        ).to_owned()
                    },

                    $(stringify!($command) => {
                        const NUM_ARGS: usize = $crate::count_idents!($($arg_name)*);
                        let arguments_vec = $arguments.clone().collect::<Vec<_>>();

                        if arguments_vec.len() != NUM_ARGS {
                            return Some(format!(
                                "\
                                ~{} takes {} arguments: {}.
                                ",
                                stringify!($command),
                                NUM_ARGS,
                                concat!($("`", stringify!($arg_name), "`"),*),
                            ))
                        }

                        if let [$($arg_name),*] = arguments_vec.as_slice() {
                            $exp
                        } else {
                            unreachable!()
                        }
                    },)*

                    _ => unreachable!(),
                }),

                _ => None,
            }
        }
    };
}