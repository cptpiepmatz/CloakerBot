# Changes and decisions

## SQLite replacement (in progress)

- Starting fresh, as requested: no JSON importer, dual-write mode, or legacy-format compatibility.
- Added SQLx 0.9 with only SQLite/Tokio, checked-query, embedded-migration, and time features. The initial schema is normalized and uses strict tables, foreign keys, nonnegative counters, bounded random intervals, and cascading guild cleanup.
- The database will live at `data/cloaker.sqlite`, using WAL, FULL synchronous commits, a five-second busy timeout, and a small connection pool. New Unix directories/files are private; Windows inherits the directory ACL. SQL statement logging is disabled for credential safety.
- Offline query metadata will be checked in so normal builds do not need a schema database. The initial migration creates fresh databases automatically; it is a schema migration, not a legacy data migration.

The following sections describe the earlier TODO/dependency work; their JSON-persistence details are superseded by this replacement.

Resolved all 33 original TODO, FIXME, and FILLME markers, each in a separate initial commit on `vibin`. No new tests were added; the existing sound-duration assertion was made precise. Live Discord behavior still needs deployment verification.

## Dependency refresh

- Selected one shared Ring TLS provider before constructing clients in both binaries. reqwest 0.13 uses `rustls-no-provider` so it does not introduce AWS-LC alongside the Discord stack's Ring provider, avoiding ambiguous-provider startup panics and an unnecessary native crypto build.

- No Rust compiler version is pinned: there is no toolchain file or `rust-version`, and both Docker builds use `rust:latest`. Kept that policy; validation uses installed stable Rust 1.99.1. The workspace already uses Rust 2024.
- Updated the full lockfile and direct requirements for current stable release lines, including Twilight 0.17, Songbird 0.6 (DAVE support), reqwest 0.13, rand 0.10, parking_lot 0.13, imageproc 0.27, axum-extra 0.12, syn 3, toml 1.1, konst 0.4, heapless 0.9, itertools 0.15, strum 0.28, and console-subscriber 0.5.
- Migrated component IDs/modal-only select requirements and RNG initialization to the updated APIs; enabled reqwest's explicit form and Rustls features.
- Twilight 0.17 still has no native role-connection client/models, but its raw request API no longer needs a patched routing enum. Removed the Git dependency patch and retained the typed adapter on upstream Twilight.
- Replaced the macro error adapter with syn's native compile-error output, removing the direct `proc-macro-error2` dependency and its old compatibility warning. Removed deprecated `std::u32` imports exposed by checks.

## Integration review

- Production validation exposed an existing environment-macro bug: all fallback assignments were gated on `dev`. Corrected the feature gates for dev/prod values and made the common fallback unconditional.

- Updated the observable cache before emitting change notifications or processing events in Songbird: a takedown owns the call mutex, so Songbird may wait on that mutex while the bot is moved. Rejection detection must see the gateway state before that wait. Also released danger-zone config reads before takedown execution and revalidated the original danger channel after queuing.

- Formatted the workspace with the existing nightly rustfmt configuration, preserving the project's import grouping, brace style, and sparse comments.

- Preserved a confirmed parry even if caller lookup/removal fails: the original target was defended, but no returned takedown was confirmed. Caller absence is checked before fetching roles; unexpected retaliation errors are logged without losing the defense statistic.

- Connected the takedown outcome boundary to real configured/fallback-channel log delivery. Logs identify called/random/danger-zone sources and counters/parries without pinging; skipped immune/absent targets stay quiet. Log failures never change gameplay results or statistics, and public logs do not expose transport errors.

- Made persistence write/sync a private temporary file before atomically replacing the destination, preserving the last complete JSON file on a failed write. This is particularly important for rotated OAuth refresh tokens. New Unix files use mode 0600; Windows inherits the data directory's ACL.

- Supervised HTTP, gateway/event, and linked-role tasks with `select!`, so a bind failure or stopped worker is reported immediately instead of waiting forever for the other workers.

- Consumed and validated the callback nonce before token exchange or any account/DM side effects, so invalid state cannot perform work before being rejected.

## Completed markers

Each row corresponds to a separate initial commit, in commit order. Follow-up integration and dependency changes have their own commits.

| # | Original location / subject | Change and decision |
| --- | --- | --- |
| 1 | `assets/quotes.toml` | Replaced random placeholders; centralized missing quotes in the existing typed compile-time assets. |
| 2 | `cloaker.rs`: success quote | Used the shared success quote. |
| 3 | `cloaker.rs`: countered quote | Used the dedicated countered quote. |
| 4 | `cloaker.rs`: failure quote | Used the failure quote while retaining the specific reason. |
| 5 | `counter.rs`: success quote | Used the shared successful-counter quote. |
| 6 | `counter.rs`: missed quote | Used the shared missed-counter quote. |
| 7 | Config random button quote | Selected enabled/disabled quotes from the resulting configuration. |
| 8 | `ranks.rs`: implement Error | Added contextual `thiserror` messages and a transparent graphics-error source. |
| 9 | `main.rs`: allowed mentions | Disabled mention parsing by default for the bot HTTP client. |
| 10 | Config command fallback channel | Implemented system → public updates → #general, checking effective view/send permissions; missing data fails closed. |
| 11 | Channel select fallback | Displayed the actual fallback after clearing the selection, releasing config locks before lookup. |
| 12 | Unset button fallback | Reused the same fallback policy and description. |
| 13 | Autocomplete handler | Enabled filtered `/help command` suggestions, capped at 25; stale/unknown requests receive empty results. |
| 14 | Split takedown function | Separated config snapshot, execution, and in-voice work; removed long-held global locks and revalidated queued targets. |
| 15 | Inner function for stats | Added one outcome boundary covering inner execution's early returns. |
| 16 | Broken tracking FIXME | Recorded successful called takedowns with a short stats write lock. |
| 17 | Track more stats | Recorded random/danger-zone successes under the bot and distinguished counters from parries with `CounterOutcome`. Fixed cross-guild punisher accumulation. |
| 18 | Other takedown errors | Retained HTTP/decoding errors and added typed audio-control failure instead of a track-stop panic. |
| 19 | Missing-permissions code | Centralized Discord 50013 (Missing Permissions) and 50001 (Missing Access). |
| 20 | Voice rejection | Checked observed bot voice state as well as Songbird's local channel before either target or caller removal. |
| 21 | Random error policy | Warned on operational errors without aborting other guilds; consumed schedules before execution and released locks. Rejected invalid intervals before sampling. |
| 22 | Precise sound duration | Updated the existing assertion to 187,840 frames / 36 kHz = 5.217777778 seconds using `Duration`. Counted samples rather than assuming 24-bit samples occupy four bytes on disk. |
| 23 | Deleted guild cleanup | Removed guild config, stats, schedule, and notification references on real removal, not temporary unavailability; shared owner records survive other references. |
| 24 | Owner notifications | Triggered v3 DMs when owners speak, with persisted history and per-owner serialization across guilds. Remembered 50007 DM refusals and split long messages below the UTF-16 limit. |
| 25 | miette errors | Rendered event diagnostics rather than panicking handlers; propagated server/task errors and kept browser error responses private. |
| 26 | Role-connection support | Added a typed adapter: neither Twilight 0.16 nor updated 0.17 has native client/models. The later upgrade removed the old routing patch. |
| 27 | Role-connection update body | Completed typed user-scoped PUT payloads with string-valued metadata and direct 64-bit global totals; no stats locks across HTTP. |
| 28 | Store OAuth tokens | Persisted user-keyed access/refresh tokens and expiry. Required Bearer tokens, nonzero lifetimes, required scopes, and redacted secret debug output. |
| 29 | Callback metadata | Published stats immediately after linking; failed metadata updates do not produce false success, and saved credentials allow retry. |
| 30 | Linking confirmation | Added explanatory DM and browser success page; blocked DMs do not fail an otherwise successful connection. |
| 31 | `prompt=consent` decision | Kept explicit consent for user-initiated linking; added secure, HTTP-only, SameSite=Lax cookies and bounded single-use five-minute nonces. |
| 32 | Linked-role lifecycle | Registered metadata at startup and synchronized every minute, refreshing expiring tokens and persisting rotations. Retried unauthorized updates once; removed revoked grants, retained transient failures for retry. |
| 33 | Owner announcement stats | Documented actual global `times_smacked`, `times_called`, and `times_countered` metadata instead of unsupported server-local criteria. |

## Deployment notes

- `data/linked_roles.json` contains credentials. Keep `data/` private through filesystem permissions; do not publish it or make unprotected backups. No encryption key is embedded in the repository.
- OAuth cookies require HTTPS; configure the Discord redirect URI and reverse proxy consistently. Restarting the bot invalidates in-flight linking nonces, so those users must start linking again.
- Linked-role connections are user-scoped. Discord server admins choose requirements against global stats; local server thresholds are not representable in this connection.
- Verify real Discord permissions, moderator moves/disconnects, linking, token refresh/revocation, and blocked DMs after deployment. No live Discord API calls were used for validation.

## Validation

| Check | Result |
| --- | --- |
| `cargo check --workspace --all-targets --locked` | Passed (default dev configuration). |
| `cargo check --workspace --all-targets --no-default-features --features prod --locked` | Passed; also exposed and verified the environment-macro fix. |
| `cargo test --workspace --locked` | Passed: 22 bot tests and 2 macro tests; no failures. |
| `cargo +nightly fmt --all -- --check` | Passed with the repository's existing configuration. |
| `git diff --check` | Passed. |
| Source/asset marker search | No remaining TODO/FIXME/FILLME/unimplemented markers in `src`, `macros`, `common`, or `assets`. |

The Windows linker still emits its import-library creation warning for the procedural macro DLL. No Rust code warnings or the previous future-incompatibility warning remain in these checks. Docker images and live Discord behavior were not exercised.
