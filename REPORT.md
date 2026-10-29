# Changes and decisions

Each original TODO, FIXME, and FILLME is handled separately. No remote API tests are added; validation uses compilation, formatting, and the existing local tests. Live Discord behavior still needs deployment verification.

## Completed items

20. Voice rejection: checked the gateway-observed bot voice channel as well as Songbird's channel after the counter window, before either target or caller removal. Songbird's local desired channel alone is insufficient to detect a moderator moving/disconnecting the bot. No failure audio is attempted in a rejected channel.

19. Permission error code: Discord's 50013 is Missing Permissions; 50001 is Missing Access. Centralized both checks for target and caller removal. A blocked returned takedown remains a parry, not a successful counter.

18. Other takedown errors: retained `ClientError` for HTTP and decoding errors and added a typed audio-control failure instead of panicking when the notification track cannot be stopped. Join failures remain separate.

17. Additional stats: recorded random/danger-zone successes under the bot, successful returned takedowns as counters, and defenses without a returned takedown as parries. `CounterOutcome` carries the removed user only when one was actually removed. Fixed global punisher aggregation to add across guilds rather than overwrite, which is required for accurate linked-role totals.

16. Broken called-takedown tracking: restored `increment_takedown` at the outcome boundary, only after successful removal, with a short stats write lock.

15. Takedown outcome boundary: made the public function observe every inner result, including early failures. This central boundary is where statistics and linked-role/log side effects can run without being skipped by `?` in execution.

14. Takedown decomposition: separated configuration snapshotting from execution and the existing in-voice phase. Removed the unused global stats guard across network/audio waits and moved the per-guild lock before target checks so queued requests revalidate the current voice state.

13. Autocomplete: implemented filtered help-command suggestions and enabled autocomplete on `/help command`. Unknown/stale requests receive an empty result rather than panicking; suggestions are limited to Discord's 25-choice maximum.

12. Log-channel unset button: used the same permission-checked fallback description as the command and channel select, avoiding three divergent fallback policies.

11. Log-channel select: showed the real fallback after clearing the selection, releasing the configuration lock before HTTP lookup.

10. Log-channel command fallback: implemented the documented system → public updates → #general order, checking effective view/send permissions. Missing cache data fails closed; the bot's own member is fetched when needed. The UI explicitly reports when no fallback is usable.

9. Mention safety: disabled mention parsing by default on the bot HTTP client. Log messages and ordinary replies can render mentions without pinging users, roles, or everyone.

8. Ranks errors: implemented `Error` with specific context and a transparent graphics-error source, matching the existing `thiserror` style.

7. Random toggle buttons: selected the existing enabled/disabled quote according to the resulting configuration.

6. `counter` miss: used the shared missed-counter quote.

5. `counter` success: used the shared successful-counter quote.

4. `cloaker` failure: used the dedicated failure quote while preserving the specific failure reason.

3. `cloaker` countered: used the dedicated countered quote.

2. `cloaker` success: used the shared success quote instead of returning a placeholder to Discord.

1. `assets/quotes.toml`: replaced random-configuration placeholder quotes and added shared takedown/counter quotes. Quotes stay in the existing typed, compile-time TOML assets rather than being scattered through handlers.
