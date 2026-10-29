# Changes and decisions

Each original TODO, FIXME, and FILLME is handled separately. No remote API tests are added; validation uses compilation, formatting, and the existing local tests. Live Discord behavior still needs deployment verification.

## Dependency refresh

- No Rust compiler version is pinned: there is no toolchain file or `rust-version`, and both Docker builds use `rust:latest`. Kept that policy; validation uses installed stable Rust 1.99.1. The workspace already uses Rust 2024.
- Updated the full lockfile and direct requirements for current stable release lines, including Twilight 0.17, Songbird 0.6 (DAVE support), reqwest 0.13, rand 0.10, parking_lot 0.13, imageproc 0.27, axum-extra 0.12, syn 3, toml 1.1, konst 0.4, heapless 0.9, itertools 0.15, strum 0.28, and console-subscriber 0.5.
- Migrated component IDs/modal-only select requirements and RNG initialization to the updated APIs; enabled reqwest's explicit form and Rustls features.
- Twilight 0.17 still has no native role-connection client/models, but its raw request API no longer needs a patched routing enum. Removed the Git dependency patch and retained the typed adapter on upstream Twilight.
- Replaced the macro error adapter with syn's native compile-error output, removing the direct `proc-macro-error2` dependency and its old compatibility warning. Removed deprecated `std::u32` imports exposed by checks.

## Integration review

- Formatted the workspace with the existing nightly rustfmt configuration, preserving the project's import grouping, brace style, and sparse comments.

- Preserved a confirmed parry even if caller lookup/removal fails: the original target was defended, but no returned takedown was confirmed. Caller absence is checked before fetching roles; unexpected retaliation errors are logged without losing the defense statistic.

- Connected the takedown outcome boundary to real configured/fallback-channel log delivery. Logs identify called/random/danger-zone sources and counters/parries without pinging; skipped immune/absent targets stay quiet. Log failures never change gameplay results or statistics, and public logs do not expose transport errors.

- Made persistence write/sync a private temporary file before atomically replacing the destination, preserving the last complete JSON file on a failed write. This is particularly important for rotated OAuth refresh tokens. New Unix files use mode 0600; Windows inherits the data directory's ACL.

- Supervised HTTP, gateway/event, and linked-role tasks with `select!`, so a bind failure or stopped worker is reported immediately instead of waiting forever for the other workers.

- Consumed and validated the callback nonce before token exchange or any account/DM side effects, so invalid state cannot perform work before being rejected.

## Completed items

33. Owner announcement stats: replaced the speculative local/global list with the three actual metadata keys and their semantics, explaining the user-scoped/global limitation and synchronization interval.

32. Linked-role lifecycle: registered metadata at startup and added a minute-based background synchronizer, including startup/restart recovery. Refreshes expiring tokens, persists rotated credentials before use, retries an unauthorized update once with refreshed credentials, and removes revoked (`invalid_grant`) grants. Per-user locks serialize relinking and refresh; transient failures leave credentials available for retry. Metadata tracks global stats, so deletion of guild stats is reflected on the next synchronization.

31. OAuth consent decision: kept explicit `prompt=consent` because linking is a user-initiated grant of stats-writing access. Hardened that flow with secure/HTTP-only/SameSite=Lax callback-scoped cookies and single-use server-side nonces expiring after five minutes (bounded pending store). Callback success clears the cookie; replayed/expired state is rejected. HTTPS termination must preserve the configured redirect URI.

30. Linking confirmation: replaced the terse placeholder DM with an explanation of stats, role requirements, and disconnecting. Added a browser success page so blocked DMs do not turn a successful connection into an HTTP failure. DM delivery is best-effort and logged separately.

29. OAuth callback metadata: published current stats immediately after linking and made update errors fail the callback rather than falsely confirming a completed connection. Persisted credentials remain available for periodic retry if Discord temporarily rejects the metadata request.

28. OAuth token persistence: persisted user-keyed access/refresh tokens and expiry, loading them on startup and flushing before callback success. Token types accept only Bearer, lifetimes are nonzero, required scopes are checked, and secret-token debug output is redacted. The new `data/linked_roles.json` contains credentials: keep `data/` private with deployment filesystem permissions and never publish or back it up unprotected; no encryption key is embedded in the repository.

27. Role-connection update implementation: completed the user-scoped PUT with a typed payload and Discord-required string metadata values. Computed global initiated/received/counter-plus-parry totals directly from stored guild stats, using 64-bit totals without sorting ranks or holding stats locks across HTTP.

26. Role-connection API adapter: inspected the pinned Twilight 0.16 client/model APIs; native support is absent. Replaced untyped metadata JSON with a typed record and numeric metadata-kind serializer while retaining Twilight's authenticated, rate-limited raw request adapter and the existing routing patch. No speculative major dependency upgrade was introduced.

25. Error rendering: added miette diagnostics for event failures with persistence/reconnection advice and rendered reports instead of panicking spawned handlers. HTTP bind/serve and task-join errors propagate to the miette-returning entry point. Remote callback errors continue returning safe HTTP statuses rather than leaking reports to browsers.

24. Owner notifications: on a human guild owner's message, sent the v3 announcement once per owner across guilds. Loaded notification history from disk, serialized concurrent sends per owner, respected previously blocked DMs, and remembered Discord's 50007 refusal rather than retrying on every message. Long announcements are split safely below Discord's UTF-16 message limit; transient failures remain retryable. No message-content intent is required.

23. Deleted guild cleanup: removed configuration, stats, random schedule, and owner-notification references only on actual removal, never temporary unavailability. Shared owner records survive while another guild refers to them. Cleanup shares the takedown lock through outcome recording, and queued takedowns reject guilds absent from cache to avoid recreating deleted stats.

22. Sound duration: updated the existing assertion to the actual 187,840 frames at 36 kHz (5.217777778 seconds), comparing `Duration` rather than truncated seconds. Duration now uses sample count/channel count/sample rate, also avoiding the incorrect four-byte assumption for packed 24-bit WAV samples. No new test was added.

21. Random-takedown failure policy: expected gameplay failures do not abort the scheduler; transport/join/audio failures are warned and other guilds still run. Due entries are consumed and locks released before executing takedowns, preventing recursive config-read deadlocks and duplicate scheduling. Invalid persisted intervals are rejected before random sampling (positive finite ordered hours, at most one year).

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
