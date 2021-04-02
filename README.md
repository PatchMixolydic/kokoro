# Kokoro
Kokoro is a Discord bot that allows you to take control of multiple virtual
users using webhooks. It's primarily meant to be used for ~~roleplay~~ putting
your favourite characters in ~~relatably depressing~~ absolutely hilarious
scenarios.

![A conversation between several characters from Touhou Project. Kokoro: "Has anyone been having any issues." Eiki: "I'm lonely [frown]". Yuuka: "nobody takes me seriously". Mima: "i feel like nobody remembers who i am". Kokoro: "I was talking about the server but do you folks need a hug."](https://github.com/PatchMixolydic/kokoro/blob/main/media/example.png?raw=true)

## Rationale
Kokoro is heavily inspired by [PluralKit](https://pluralkit.me/). However, there
are a few key differences between Kokoro and PluralKit:

* PluralKit is primarily intended as an accessibility tool, while Kokoro is
  geared towards more lighthearted uses. Kokoro might still be useful for
  plural users, but this is not guaranteed.
* Kokoro is significantly more limited than PluralKit. For example, PluralKit
  allows for several kinds of [proxy tags], including circumscribed text,
  prefixes, and postfixes. Kokoro currently only allows for prefixes.
* Kokoro is a work-in-progress, while PluralKit is a mature product.
* Kokoro does not currently have commands to edit your characters; you must
  edit the database manually. This will likely be fixed in due time.
* PluralKit is written in C#, while Kokoro is written in Rust. This may make
  deployment a bit easier, especially on not-Windows.

[proxy tags]: https://pluralkit.me/start/#set-some-proxy-tags

## Building
To build Kokoro, you will need to install a
[Rust development environment][rust] and [sqlx-cli] with the `sqlite` feature.

First, clone the repository:
```console
$ git clone https://github.com/PatchMixolydic/kokoro.git
...
$ cd kokoro
```

Next, create a file named `.env` with the following format. Be sure to replace
the placeholders with useful values.
```
BOT_TOKEN=your discord bot's token
BOT_USER_ID=your discord bot's client ID
DATABASE_URL=sqlite:database_name_here.db
```

Run `sqlx database create` to create the database.
```
$ sqlx database create
$ ls database_name_here.db
database_name_here.db
```

Now, you should be able to run `cargo build` to generate the target executable
in `./target/debug` (you can also run `cargo build --release`, which will
create an optimized executable in `./target/release`).
```
$ cargo build
...
    Finished dev [unoptimized + debuginfo] target(s) in 0.09s
$ ls target/debug
build  deps  examples  incremental  kokoro  kokoro.d
```

You can now run Kokoro using `cargo run` (or `cargo run --release`):
```
$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.09s
     Running `target/debug/kokoro`
Connected on shard 0.
```

[rust]: https://www.rust-lang.org/learn/get-started
[sqlx-cli]: https://github.com/launchbadge/sqlx/tree/master/sqlx-cli#with-rust-toolchain

## Usage
To create a character, use the `~create [name] [avatar_url] [prefix]` command.
Note that arguments can be surrounded with quotes to allow spaces.

```
PatchMixolydic: ~create "Hina Kagiyama" https://example.com/hina.png hi:
Kokoro: Created Hina Kagiyama successfully.
```

You can list all your characters and their prefixes using `ls`:
```
PatchMixolydic: ~ls
Kokoro: Hina Kagiyama (prefix hi:)
        Patchouli (prefix pat:)
        Junko (prefix jun:)
```

To post as a character, send a message starting with that character's prefix:
```
PatchMixolydic: hi: hi everyone!
```

Kokoro will take your message (without the prefix) and send it as your
character via a webhook. Note that leading and trailing whitespace is stripped.
The bot will then delete your original message.
```
Hina Kagiyama: hi everyone!
```

Currently, there is no way to edit or delete characters. This may be fixed in
a future update.
