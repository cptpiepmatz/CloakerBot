<p align="center">
  <picture>
    <img width="665" src="./assets/graphics/user_rank/banner.webp" alt="Cloaker Takedown banner">
  </picture>
</p>
<h1 align="center">CloakerBot</h1>
<p align="center">
  <b>🥷 A Discord bot that will kick your friends in the face.</b>
</p>

<br>

<div align="center">

[![License](https://img.shields.io/badge/License-PolyForm_Noncommercial_1.0.0-blue?style=for-the-badge)](./LICENSE)
[![Version](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2Fcptpiepmatz%2FCloakerBot%2Fmain%2FCargo.toml&query=%24.workspace.package.version&style=for-the-badge&label=version&prefix=v&color=brightgreen)](https://github.com/cptpiepmatz/CloakerBot/tags)
[![Rust](https://img.shields.io/badge/Built_with-Rust-D34516?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Add to Discord](https://img.shields.io/badge/Add_to_Discord-5865F2?style=for-the-badge&logo=discord&logoColor=white)](https://cloakerbot.cptpiepmatz.de/invite)

</div>

## About

**CloakerBot** brings the Cloaker from
[PAYDAY 2](https://store.steampowered.com/app/218620/PAYDAY_2/) into your Discord
voice channels. Pick a friend, send in the bot, and enjoy the familiar
*WULULULULU* before they get disconnected from voice. Not kicked from the server,
just from the conversation.

Quick enough? Counter the attack and send it back at whoever called it.
Takedowns, unfortunate victims, and successful counters all have their own stats
and ranks, so there is something to show for the chaos.

<!-- TODO: Upload the WebM demo to GitHub and put its attachment URL here. -->
> **Demo:** A video of CloakerBot in action will go here.

## Add to your Discord

<p align="center">
  <a href="https://cloakerbot.cptpiepmatz.de/invite">
    <img height="60" src="https://img.shields.io/badge/Invite_CloakerBot-Add_to_Discord-5865F2?style=for-the-badge&amp;logo=discord&amp;logoColor=white" alt="Invite CloakerBot to your Discord server">
  </a>
</p>

Invite the bot to your server, then try `/cloaker` with a friend who is in a
voice channel. The invite requests the permissions needed to see channels,
send messages, join and speak in voice, and disconnect members.

## Features

### Slash commands

Type `/` in chat to find the bot's commands. If you are new to them, see
Discord's [Slash Commands FAQ][slash-commands].

| Command                  | What it does                                                                                                                                   |
|--------------------------|------------------------------------------------------------------------------------------------------------------------------------------------|
| `/cloaker target:<user>` | Send the Cloaker into your target's voice channel, play the sounds, and disconnect them with a parting quote. The main attraction.             |
| `/counter`               | Turn an incoming takedown back on its caller. Against a random takedown, send the bot away instead. Danger-zone takedowns cannot be countered. |
| `/ranks [user]`          | Show your rank profile, or someone else's: punisher, victim, and counter ranks, server and global counts, plus badges.                         |
| `/config`                | Open the server's private configuration panel. Requires **Manage Server**.                                                                     |
| `/help`                  | Browse the in-Discord help for commands and features.                                                                                          |

### Context commands

No need to type every time: right-click a user or message and open **Apps** to
use `cloaker` or `counter`. The `cloaker` action targets the selected user or the
message's author; `counter` counters the attack coming at you.

Discord explains these shortcuts under
[User and Message Commands][context-commands].

### Linked roles

Turn your track record into server roles using
[Discord's linked roles][linked-roles]. Connect your account through
[CloakerBot's linking page](https://cloakerbot.cptpiepmatz.de/linked-role), then
use these stats as requirements for a linked role:

| Stat | What it counts |
| --- | --- |
| **Times Smacked** | Successful takedowns you initiated globally. |
| **Times Called** | Times you were taken down globally. |
| **Times Countered** | Successful counters and parries globally. |

Each requirement is a minimum count. For example, you could make a role for
members who have countered at least ten takedowns. These are global totals,
not just activity in your server.

## Configuration

Use `/config` to see and change the server's settings, and `/help` for the
details. Random takedowns, danger zones, immunity roles, the log channel, and
bot targeting are all configured there.

To control who can use the bot and in which channels, open **Server Settings →
Apps → Integrations → CloakerBot** on desktop. Discord's
[Command Permissions guide][command-permissions] covers app-wide and
per-command permissions.

## Support

Questions, trouble, or just want to talk about the bot? Join the
[support server](https://cloakerbot.cptpiepmatz.de/discord).

## Hosting

The public bot is self-hosted, and you can host your own instance too. You will
need your own application in the
[Discord Developer Portal](https://discord.com/developers/applications), its bot
token and OAuth2 client secret, and a public HTTPS address for account linking.

Configuration comes from environment variables; no `.env` file is loaded.
Some values are embedded **at build time**, so running the existing binary with
different values will not change them. Set your own application ID and redirect
URI before compiling rather than using the public bot's defaults.

### Build-time variables

| Variable                  | Default                                                                                                          | Purpose                                                                                                       |
|---------------------------|------------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------|
| `DISCORD_CLIENT_ID`       | `1369284129031458906`; `772259037580361729` with `--cfg prod`                                                    | Your application's **Application ID**, also used as the bot user ID. This is an ID, not a token.              |
| `HTTP_PORT`               | `25625`                                                                                                          | Port for the HTTP server, including invites and account linking. It spells **cloak** on a T9 keypad.          |
| `HTTP_HOST`               | `0.0.0.0`                                                                                                        | Address the HTTP server binds to. Use `127.0.0.1` to listen only locally.                                     |
| `DISCORD_REDIRECT_URI`    | `https://dev.cloakerbot.cptpiepmatz.de/callback`; `https://cloakerbot.cptpiepmatz.de/callback` with `--cfg prod` | Public OAuth2 callback URL. Register the exact same URL under **OAuth2 → Redirects** in the Developer Portal. |
| `BACKUP_INTERVAL_SECONDS` | `300`                                                                                                            | Seconds between database snapshots. Must be between `1` and `86400`, inclusive.                               |

The `prod` configuration is a custom compiler flag, independent of Cargo's
release profile. `cargo build --release` alone still uses the first defaults
above. Explicit environment values override either set of defaults.

Install [Rust and Cargo](https://rustup.rs/) (Rust 1.95.0 or newer), a C/C++ build
toolchain, and CMake. From the repository root, for example in a POSIX shell:

```sh
DISCORD_CLIENT_ID="your-application-id" \
DISCORD_REDIRECT_URI="https://your-bot.example/callback" \
cargo build --locked --release
```

Replace the placeholders with your application's values. Sounds, graphics,
and database migrations are embedded in the binary.

### Runtime variables

| Variable                | Default                                  | Purpose                                                                                                                                                                            |
|-------------------------|------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `DISCORD_TOKEN`         | None; required                           | Bot token from the application's **Bot** page in the Developer Portal.                                                                                                             |
| `DISCORD_CLIENT_SECRET` | None; required                           | OAuth2 client secret from the same application. Used for linked-role authorization.                                                                                                |
| `COOKIE_SECRET`         | None; required                           | Secret used to sign OAuth2 state cookies. Supply a cryptographically random value of at least **64 bytes** and keep it private. This is generated by you, not provided by Discord. |
| `RUST_LOG`              | `warn,CloakerBot=info` in release builds | Log filter. Debug builds default to `warn,CloakerBot=debug,songbird=debug`.                                                                                                        |
| `LOG_FORMAT`            | `text`                                   | Log output format: `text` or `json`.                                                                                                                                               |

Set the required runtime variables through your shell or service manager, then
start the binary:

```sh
./target/release/CloakerBot
```

On Windows, run `target\release\CloakerBot.exe` instead. Keep credentials out of
source control and logs.

The bot creates `data/cloaker.sqlite` relative to its working directory and
applies migrations automatically. Keep that directory writable and persistent.
A periodically replaced snapshot is stored at `data/backup/cloaker.sqlite`;
copy snapshots to separate storage if you need a backup history. The database
contains linked-account OAuth2 credentials, so protect both it and its backups.

Put the HTTP server behind an HTTPS reverse proxy. Linked-role authorization
uses Secure cookies and needs HTTPS. In the Developer Portal, set the linked
role verification URL to `https://your-bot.example/linked-role` and the OAuth2
redirect to the `/callback` URL you compiled in. Your own instance's `/invite`
route generates an invite for the application ID you built with.

### Docker Compose

The [Dockerfile](./Dockerfile) supports `amd64` and `arm64` builds. Pass the
build-time variables above through **`build.args`**, and runtime variables
through **`environment`**. Setting a build-time variable in `environment` does
not change the compiled bot. Never pass tokens or secrets as build arguments.

For example, save this as `compose.yaml` alongside your reverse proxy's Compose
configuration:

```yaml
services:
  cloakerbot:
    build:
      context: https://github.com/cptpiepmatz/CloakerBot.git#main
      args:
        BUILDKIT_CONTEXT_KEEP_GIT_DIR: "1" # keep .git so the build can detect the source branch
        DISCORD_CLIENT_ID: "${DISCORD_CLIENT_ID:?Set your application ID}"
        DISCORD_REDIRECT_URI: "https://your-bot.example/callback"
        HTTP_HOST: "0.0.0.0"
        HTTP_PORT: "25625"
        BACKUP_INTERVAL_SECONDS: "300"
    environment:
      DISCORD_TOKEN: "${DISCORD_TOKEN:?Set your bot token}"
      DISCORD_CLIENT_SECRET: "${DISCORD_CLIENT_SECRET:?Set your OAuth2 client secret}"
      COOKIE_SECRET: "${COOKIE_SECRET:?Set a random secret of at least 64 bytes}"
    volumes:
      - cloakerbot-data:/cloaker-bot/data
    expose:
      - "25625"
    restart: unless-stopped

volumes:
  cloakerbot-data:
```

Replace the example domain and supply the interpolated values through your
shell or a private Compose `.env` file. Compose reads that file for variable
substitution; the bot itself does not. Keep it out of source control.

Remote Git builds need
`BUILDKIT_CONTEXT_KEEP_GIT_DIR: "1"` to retain `.git`. The Dockerfile copies
that metadata into the build stage so the bot can detect its source branch;
it is not included in the final image. For a local checkout, use `context: .`
instead and keep its `.git` directory available in the build context.

Start with `docker compose up -d --build`. Rebuild after changing `build.args`;
recreating the container alone is not enough. The Dockerfile also accepts
`PROD: "1"` as a build argument to enable `--cfg prod`, but explicit values
still override its defaults.

The example assumes a containerized reverse proxy on the same Docker network.
Point it at `http://cloakerbot:25625` and terminate HTTPS there. `expose` does
not publish the port on the host. If your proxy runs on the host instead, use
`ports: ["127.0.0.1:25625:25625"]` in place of `expose`. Keep `HTTP_HOST` set
to `0.0.0.0` inside the container so the proxy can reach it. If you change the
compiled port, update the Compose port configuration and proxy upstream too.

### Container helper script

For local builds and runs, [images.nu](./images.nu) provides a
[Nushell](https://www.nushell.sh/) helper. Run it from the repository root with
Podman or Docker installed; it prefers Podman when both are available.

```sh
nu images.nu build
nu images.nu run

# For ARM64 instead:
nu images.nu build arm64
nu images.nu run arm64
```

Images are tagged with the package version and current Git commit. The run
command uses that tag, reads `DISCORD_TOKEN`, `DISCORD_CLIENT_SECRET`, and
`COOKIE_SECRET` from the environment, and prompts with hidden input for any
missing or empty values. It mounts local `./data` into the container and
publishes port `25625` on **all IPv4 interfaces**. Keep network access restricted
as appropriate and use an HTTPS reverse proxy for account linking.

The helper's build commands use the Dockerfile's defaults; they do not forward
build-time environment variables as build arguments. For a custom application
ID or other build-time settings, use Compose as shown above, or build directly
with Docker or Podman using `--build-arg`. Run `nu images.nu --help` to list the
available commands.

## Contributing

If something is broken or you have an idea,
[open an issue](https://github.com/cptpiepmatz/CloakerBot/issues).
Pull requests are welcome too, but this is a personal project: do not expect
much activity or quick reviews.

## License

Original code and contributions are licensed under the
[PolyForm Noncommercial License 1.0.0](./LICENSE).
Third-party audio, quotations, and artwork are excluded from that license;
see [Credits](./CREDITS.md) for attribution and rights information.

[slash-commands]: https://support-apps.discord.com/hc/en-us/articles/26501837786775-Slash-Commands-FAQ
[context-commands]: https://support.discord.com/hc/en-us/articles/21334461140375-Using-Apps-on-Discord
[linked-roles]: https://support.discord.com/hc/en-us/articles/10388356626711-Connections-Linked-Roles-Admins
[command-permissions]: https://support-apps.discord.com/hc/en-us/articles/26501869403159-Command-Permissions
