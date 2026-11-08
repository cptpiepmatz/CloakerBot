# CloakerBot Privacy Policy

**Last updated:** November 8, 2026

## Who operates the service

Tim "Piepmatz" Hesse operates the hosted CloakerBot service and its related HTTP
endpoints. For privacy questions or data requests, join the [CloakerBot support
Discord server][support] and ask to contact the operator privately. Do not post
tokens or other sensitive information in public channels or GitHub issues.

This policy describes the operator's hosted service. Independently hosted copies
of the open-source bot are the responsibility of their own operators.

## Data processed and why

- **Discord information and interactions:** The bot receives Discord events and
  command/component/modal inputs, including user, server, channel, role, and
  interaction IDs. It processes server/member information, profiles, permissions,
  and voice-channel membership to execute commands, check immunity, and operate
  voice features. Discord information is also cached in memory during operation.

- **Server configuration:** The database stores server IDs, log-channel IDs,
  danger-zone channel IDs, immunity-role IDs, bot-targeting and random-event
  settings, and scheduled random-event times so the bot can remember its settings.

- **Statistics:** The database stores per-server counts associated with caller
  and target user IDs for takedowns and counters, and user IDs with parry counts.
  These support counters, ranks, and global statistics across servers.

- **Owner notifications:** The bot uses message-event metadata to recognize
  activity by a server owner and send service announcements. It stores the owner's
  user ID, notification preference, and announcement progress. Message text is
  not needed for this feature and is not written to the bot's database.

- **Optional linked roles:** If you authorize a Discord connection, the bot stores
  your user ID, OAuth access and refresh tokens, and token expiry. It requests
  `identify` and `role_connections.write` to associate your account and send your
  global takedown, received-takedown, and counter/parry totals to Discord for linked
  roles. These credentials allow updates without asking you to sign in each time.

- **Operational diagnostics:** Application logs can include timestamps, user,
  server, channel, and interaction IDs, action outcomes, and error details. They
  help troubleshoot failures and maintain the service. The HTTP service and its
  reverse proxy also process connection/request information needed to serve
  requests, such as IP addresses and requested URLs; any proxy logging depends
  on its configuration.

CloakerBot does not record or transcribe voice conversations. It deafens itself
when joining voice channels and plays bundled sounds. It does not maintain a
database of message text or use data for advertising or sale.

## Cookies

The optional linked-role sign-in flow sets a signed security cookie named `state`
to verify the OAuth callback. It has a five-minute lifetime and is removed after
successful verification. It is not an advertising or analytics cookie.

## Where data goes and who can see it

The service currently runs on a self-hosted Raspberry Pi behind a reverse proxy.
Its SQLite database, local database snapshots, and Docker-managed application
logs are stored on the operator's device. The operator can access these for
administration, support, and handling data requests. The database and its snapshots
include linked-role credentials when accounts are connected.

Discord receives the API requests, messages, voice playback, and optional
linked-role updates needed to provide these features. Commands can display
another user's statistics, including global totals. Takedown notifications can
identify callers, targets, channels, and outcomes in a configured log channel or
an available fallback channel. Visibility depends on Discord channel permissions
and the type of command response. These messages are stored by Discord, separately
from the bot's database.

Opening policy pages hosted on GitHub sends your browser to GitHub. Discord and
GitHub process data under their own privacy policies: [Discord][discord-privacy]
and [GitHub][github-privacy].

Data may also be disclosed if required by law. If the service moves to a hosting
provider, this policy will be updated to describe the new hosting arrangement.

## Retention and deletion

- **Server records:** Settings, schedules, and per-server statistics are kept
  while the bot serves a server. The bot deletes these database records when it
  receives a permanent server-removal event from Discord. A member leaving a
  server does not automatically delete their statistics.

- **Linked-role credentials:** These are stored while the connection is maintained.
  The bot deletes them when a token refresh identifies the authorization as
  revoked. Revocation may not be detected immediately. Removing the bot from a
  server does not by itself remove a separate account connection.

- **Owner preferences:** Notification preferences and announcement progress have
  no automatic expiry and are not automatically removed with a server.

- **Logs and backups:** There is currently no fixed maximum retention period.
  Logs remain locally according to Docker settings and manual cleanup. The bot
  normally replaces its local database snapshot every five minutes, but failed
  snapshots may leave an older copy in place. Other retained backup copies and
  logs are cleaned up manually, including when storage space is needed; space
  availability alone does not guarantee a deletion date.

Deleting an active database record does not immediately erase older logs,
backup copies, or messages already sent to Discord. Contact the operator if your
request also concerns those copies. Database removal does not automatically
remove role metadata already sent to Discord.

## Your choices and requests

You can revoke optional authorization through Discord's account settings. Server
administrators can change settings or remove the bot, and owners can disable
owner-announcement DMs using the control provided in those messages.

Contact the operator privately through the support Discord server to request
access to, correction of, or deletion of data associated with your Discord user
ID. Server administrators can also request removal of server data. Provide the
relevant IDs; the operator may need to verify account ownership or server
authority. There is currently no general self-service data-deletion command.
Continued use of the bot can create new records after deletion.

Depending on applicable law, you may also have rights to restriction, objection,
data portability, and to complain to a data protection authority. Requests will
be handled subject to applicable legal requirements.

## Changes

Updates will be published here with a revised date. Material changes will also
be announced in the support Discord server.

[support]: https://cloakerbot.cptpiepmatz.de/discord
[discord-privacy]: https://discord.com/privacy
[github-privacy]: https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement
