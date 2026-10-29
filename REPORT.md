# Changes and decisions

Reviewed against the current source, schema, manifests, Dockerfiles, tests, and commit history. The sections below describe current behavior unless explicitly marked as historical; verification is local, not a claim of live Discord or container validation.

## SQLite replacement

- Starting fresh, as requested: no JSON importer, dual-write mode, or legacy-format compatibility.
- Added SQLx 0.9 with only SQLite/Tokio, checked-query, embedded-migration, and time features. The initial schema is normalized and uses strict tables, foreign keys, nonnegative counters, bounded random intervals, and cascading guild cleanup.
- The database lives at `data/cloaker.sqlite`, using WAL, FULL synchronous commits, a five-second busy timeout, and a four-connection pool. Directory/file creation uses normal filesystem defaults; SQLite creates the database itself, without platform-specific permission handling. SQL statement logging is disabled for credential safety.
- Offline query metadata is checked into `.sqlx`, with offline mode enabled in `.cargo/config.toml`, so normal builds need no schema database or SQLx CLI. The initial migration creates fresh databases automatically before workers start; it is a schema migration, not a legacy data migration.

- Replaced all persistent state maps and the half-second JSON flush worker with a single database handle. Config reads are on demand; edits use short `BEGIN IMMEDIATE` transactions so concurrent component changes cannot overwrite one another. No transaction is held across Discord requests or playback.
- Takedown/counter/parry writes use atomic SQL increments. Ranking calculations keep their existing tested Rust read model, loaded as an owned, consistent snapshot per request rather than a synchronized cache. Linked-role totals are calculated directly in SQL.
- Discord IDs are stored as decimal text, preserving the full unsigned 64-bit range. SQLite counters/aggregates use signed 64-bit integers, and linked-role totals are checked when converted to Rust `u64`. The existing ranking read model still uses `u32` counts; this change did not make ranks arbitrarily large or move their calculations into SQL.
- OAuth credential writes and owner-notification acknowledgements commit immediately. Existing per-user locks continue to serialize refresh/relink and announcement sends. Database errors propagate through the existing diagnostic boundaries; failure to record an already completed takedown is logged without misreporting the gameplay result.
- Random schedules are claimed transactionally, including a config recheck; changing random settings invalidates the existing schedule in the same transaction. Guild deletion cascades through config, stats, schedules, and owner references while preserving linked accounts and owners still referenced by another guild.
- Added two focused local SQLite tests for the replacement: fresh initialization/reopening, full-range Discord ID roundtrips, credential replacement/removal, totals/rank snapshots, schedule claims, rejected config edits preserving prior values, shared-owner cleanup, concurrent increments/edits, and schema constraints. These test local storage operations, not real Discord token rotation/revocation or crash recovery.
- Updated both Docker build contexts to include embedded migrations, offline SQLx metadata, build script, and workspace crates. Removed the stale single-crate bootstrap build and unnecessary token-file copy; builds use `--locked` and need no schema database.

### Snapshots and backups

- After database initialization, a supervised background worker attempts its first snapshot immediately, then on a five-minute interval, skipping missed ticks. Publication is asynchronous: startup does not wait for a completed snapshot. Set runtime `BACKUP_INTERVAL_SECONDS` to an integer from 1 through 86400 to change the interval. Snapshot failures are warned and retried on the next tick without stopping gameplay; until the first successful snapshot there is no backup file.
- Snapshots use SQLite's `VACUUM INTO` (a consistent online snapshot, supported directly through SQLx without unsafe backup-API bindings), then sync the temporary file and atomically replace the published snapshot. Generation is serialized. Failed creation/replacement leaves the previous published file intact, including Windows sharing conflicts with a copying tool. This regenerates the snapshot each interval rather than doing incremental backup.
- Configure the zip backup to include `data/backup/cloaker.sqlite` and exclude the live `data/cloaker.sqlite`, its `-wal`/`-shm` sidecars, and `data/backup/*.tmp`. The published file is self-contained and can be copied at any time; it contains plaintext credentials. A tool that denies replacement while reading can delay publication on Windows, but the older snapshot remains available.
- For restore, stop the bot and restore the published snapshot as `data/cloaker.sqlite`, without stale live `-wal`/`-shm` files. Changes after the snapshot's consistent read point are not included; snapshot duration and failures can extend the recovery window beyond the configured interval. Credential changes do not trigger extra snapshots. Restoring older rotated OAuth credentials can require relinking.
- Added a local snapshot test covering standalone restore from a running WAL database, integrity checking, failed temporary-path setup/replacement preserving the previous file, Windows reader sharing conflicts, and successful subsequent refresh. It does not simulate power loss or invoke a real zip backup tool.

### Running and developing

- Both binaries read the bot token only from the runtime `DISCORD_TOKEN` environment variable. The bot also reads `DISCORD_CLIENT_SECRET` and `COOKIE_SECRET` from the runtime environment. There is no token-file fallback or automatic secrets-file loading, and Docker builds do not copy token files. Existing `.token`/`dev.token` files are left untouched and remain ignored by Git.
- Run normally; startup creates `data/cloaker.sqlite` and applies the embedded `migrations/`. Old JSON files are ignored, not imported or deleted. The entire `data/` directory remains ignored by Git.
- The database and its sidecars/backups contain plaintext OAuth credentials; do not publish them. In containers, mount persistent storage at `/cloaker-bot/data`.
- Use the published snapshot for live file-copy backups. To copy the live database directly instead, stop the bot first or use SQLite-aware backup tooling. SQLite improves local durability but cannot atomically commit Discord token rotation, a voice removal, or a DM delivery together with a database write.
- After editing queries or adding schema migrations, use a disposable schema database and regenerate offline metadata (PowerShell):

  ```powershell
  New-Item -ItemType Directory -Force target | Out-Null
  $env:DATABASE_URL = 'sqlite:target/sqlx-schema.sqlite'
  $env:SQLX_OFFLINE = 'false'
  cargo sqlx database setup
  cargo sqlx prepare --workspace -- --all-targets
  # commit migrations and .sqlx; return ordinary builds to offline mode
  Remove-Item Env:DATABASE_URL, Env:SQLX_OFFLINE
  ```

  This maintenance workflow requires `sqlx-cli` 0.9 with SQLite support. Add new migrations rather than editing migrations applied to a deployed database. `cargo sqlx prepare --check --workspace -- --all-targets` verifies metadata against the schema database.

  Schema-dependent application queries use checked SQLx macros. The schema-independent `VACUUM INTO` maintenance command and test-only inspection/constraint statements use runtime queries, so they do not generate offline metadata.

### SQLite validation

All checks in this table were rerun during this report audit against the current code (2026-10-29).

| Check | Result |
| --- | --- |
| `cargo check --workspace --all-targets --locked` | Passed offline, without a schema database. |
| `cargo check --workspace --all-targets --no-default-features --features prod --locked` | Passed offline. |
| `cargo test --workspace --locked` | Passed: 25 bot tests (including three storage tests) and 2 macro tests; no failures. |
| `cargo sqlx prepare --check --workspace -- --all-targets` | Passed against a disposable database created from `migrations/`. |
| `cargo +nightly fmt --all -- --check` | Passed. |
| Marker/legacy-persistence search | No remaining TODO/FIXME/FILLME/unimplemented markers, `PersistentData`, or periodic JSON flush worker in source/assets. The periodic SQLite snapshot worker is intentional. |
| `git diff --check` | Passed. |

Only the existing Windows macro-DLL linker warning remains. Docker images and live Discord/OAuth flows were not exercised.

The following sections summarize the earlier TODO/dependency work and note where SQLite superseded an implementation detail.

Resolved all 33 original TODO, FIXME, and FILLME markers, each in a separate initial commit on `vibin`. That original phase added no tests; the existing sound-duration assertion was made precise. The SQLite phase above adds three targeted storage tests. Live Discord behavior still needs deployment verification.

## Dependency refresh

- Selected one shared Ring TLS provider before constructing clients in both binaries. reqwest 0.13 uses `rustls-no-provider` so it does not introduce AWS-LC alongside the Discord stack's Ring provider, avoiding ambiguous-provider startup panics and an unnecessary native crypto build.

- No Rust compiler version is pinned: there is no toolchain file or `rust-version`, and both Docker builds use `rust:latest`. Kept that policy; validation uses installed stable Rust 1.99.1. The workspace already uses Rust 2024.
- Updated the full lockfile and direct requirement lines, including Twilight 0.17, Songbird 0.6 (DAVE support), reqwest 0.13, rand 0.10, parking_lot 0.13, imageproc 0.27, axum-extra 0.12, syn 3, toml 1.1, konst 0.4, heapless 0.9, itertools 0.15, strum 0.28, and console-subscriber 0.5. These describe the completed refresh, not a guarantee that every requirement always tracks the newest available release.
- Migrated component IDs/modal-only select requirements and RNG initialization to the updated APIs; enabled reqwest's explicit form and Rustls features.
- Twilight 0.17 still has no native role-connection client/models, but its raw request API no longer needs a patched routing enum. Removed the Git dependency patch and retained the typed adapter on upstream Twilight.
- Replaced the macro error adapter with syn's native compile-error output, removing the direct `proc-macro-error2` dependency and its old compatibility warning. Removed deprecated `std::u32` imports exposed by checks.

## Integration review

- Production validation exposed an existing environment-macro bug: all fallback assignments were gated on `dev`. Corrected the feature gates for dev/prod values and made the common fallback unconditional.

- Updated the observable cache before emitting change notifications or processing events in Songbird: a takedown owns the call mutex, so Songbird may wait on that mutex while the bot is moved. Rejection detection must see the gateway state before that wait. Also released danger-zone config reads before takedown execution and revalidated the original danger channel after queuing.

- Formatted the workspace with the existing nightly rustfmt configuration, preserving the project's import grouping, brace style, and sparse comments.

- Preserved a confirmed parry even if caller lookup/removal fails: the original target was defended, but no returned takedown was confirmed. Caller absence is checked before fetching roles; unexpected retaliation errors are logged without losing the defense statistic.

- Connected the takedown outcome boundary to real configured/fallback-channel log delivery. Fallback lookup applies when no channel is configured; an unusable explicit channel is skipped rather than rerouted. Logs identify called/random/danger-zone sources and counters/parries without pinging; skipped immune/absent targets stay quiet. Log failures never change gameplay results or statistics, and public logs do not expose transport errors.

- Historical, now removed: JSON persistence used a synced temporary file and atomic replacement. SQLite transactions have replaced that code. The app no longer sets explicit Unix permission modes; database and snapshot files use normal filesystem defaults.

- Supervised HTTP, gateway/event-loop, linked-role, and snapshot workers with `select!`, so a bind failure or stopped top-level worker is reported instead of waiting forever for the others. Individual event-handler tasks are spawned separately; this is not supervision of every handler panic.

- Consumed and validated the callback nonce before token exchange or any account/DM side effects, so invalid state cannot perform work before being rejected.

## Completed markers

Each row corresponds to a separate initial commit, in commit order. Follow-up integration, dependency, SQLite, and snapshot changes have their own commits. Descriptions reflect the current implementation where it has replaced the original one.

| # | Original location / subject | Change and decision |
| --- | --- | --- |
| 1 | `assets/quotes.toml` | Replaced random placeholders; centralized missing quotes in the existing typed compile-time assets. |
| 2 | `cloaker.rs`: success quote | Used the shared success quote. |
| 3 | `cloaker.rs`: countered quote | Used the dedicated countered quote. |
| 4 | `cloaker.rs`: failure quote | Used the failure quote while retaining the specific reason. |
| 5 | `counter.rs`: success quote | Used the shared successful-counter quote. |
| 6 | `counter.rs`: missed quote | Used the shared missed-counter quote. |
| 7 | Config random button quote | Selected enabled/disabled quotes from the resulting configuration. |
| 8 | `interaction/application_command/ranks.rs`: implement Error | Added `thiserror` messages for missing guild/author and a transparent graphics-error source. The graphics error now also carries database failures. |
| 9 | `main.rs`: allowed mentions | Disabled mention parsing by default for the bot HTTP client. |
| 10 | Config command fallback channel | Implemented system → public updates → #general, checking effective view/send permissions; missing data fails closed. |
| 11 | Channel select fallback | Displays the actual fallback after clearing the selection. The config change now commits to SQLite before fallback lookup; no transaction is held across that lookup's HTTP request. |
| 12 | Unset button fallback | Reused the same fallback policy and description. |
| 13 | Autocomplete handler | Enabled filtered `/help command` suggestions, capped at 25; stale/unknown requests receive empty results. |
| 14 | Split takedown function | Separated config snapshot, execution, and in-voice work; removed long-held global locks and revalidated queued targets. |
| 15 | Inner function for stats | Added one outcome boundary covering inner execution's early returns. The outer missing-guild and configuration-load checks return before that boundary. |
| 16 | Broken tracking FIXME | Recorded successful called takedowns; the original in-memory stats write lock has since been replaced by a committed SQL increment. |
| 17 | Track more stats | Recorded random/danger-zone successes under the bot and distinguished counters from parries with `CounterOutcome`. Fixed cross-guild punisher accumulation. |
| 18 | Other takedown errors | Retained HTTP/decoding errors and added typed audio-control failure instead of a track-stop panic. |
| 19 | Missing-permissions code | Centralized Discord 50013 (Missing Permissions) and 50001 (Missing Access). |
| 20 | Voice rejection | Checked observed bot voice state as well as Songbird's local channel before either target or caller removal. |
| 21 | Random error policy | Warns on per-takedown client/database/join/audio failures and continues with other queued guilds. Scheduler database-query failures still abort that check and are reported by the event handler. Claims schedules before execution and rejects invalid intervals before sampling. |
| 22 | Precise sound duration | Updated the existing assertion to `Duration::from_nanos(5_217_777_778)` (187,840 frames / 36 kHz, rounded to nanoseconds). Counted samples rather than assuming 24-bit samples occupy four bytes on disk. |
| 23 | Deleted guild cleanup | Removed guild config, stats, schedule, and notification references on real removal, not temporary unavailability; shared owner records survive other references. |
| 24 | Owner notifications | Triggered v3 DMs when owners speak, with persisted history and per-owner serialization across guilds. Remembered 50007 DM refusals and split long messages below the UTF-16 limit. |
| 25 | miette errors | Renders returned event errors as reports; propagates top-level server/task errors and keeps callback HTTP error responses private. This does not catch arbitrary handler panics. |
| 26 | Role-connection support | Added a typed adapter: neither Twilight 0.16 nor updated 0.17 has native client/models. The later upgrade removed the old routing patch. |
| 27 | Role-connection update body | Completed typed user-scoped PUT payloads with string-valued metadata and direct 64-bit global totals; no stats locks across HTTP. |
| 28 | Store OAuth tokens | Persisted user-keyed access/refresh tokens and expiry. Required Bearer tokens, nonzero lifetimes, required scopes, and redacted secret debug output. |
| 29 | Callback metadata | Published stats immediately after linking; failed metadata updates do not produce false success, and saved credentials allow retry. |
| 30 | Linking confirmation | Added explanatory DM and browser success page; blocked DMs do not fail an otherwise successful connection. |
| 31 | `prompt=consent` decision | Kept explicit consent for user-initiated linking; added secure, HTTP-only, SameSite=Lax cookies and bounded single-use five-minute nonces. |
| 32 | Linked-role lifecycle | Registered metadata at startup and synchronized every minute, refreshing expiring tokens and persisting rotations. Retried unauthorized updates once; removed revoked grants, retained transient failures for retry. |
| 33 | Owner announcement stats | Documented actual global `times_smacked`, `times_called`, and `times_countered` metadata instead of unsupported server-local criteria. |

## Deployment notes

- Credentials are stored in `data/cloaker.sqlite` and its snapshots/sidecars, not `data/linked_roles.json`. Files are not encrypted by the app; do not publish them. Legacy JSON and token files are left untouched but are not loaded.
- OAuth cookies require HTTPS; configure the Discord redirect URI and reverse proxy consistently. Restarting the bot invalidates in-flight linking nonces, so those users must start linking again.
- Linked-role connections are user-scoped. Discord server admins choose requirements against global stats; local server thresholds are not representable in this connection.
- Verify real Discord permissions, moderator moves/disconnects, linking, token refresh/revocation, and blocked DMs after deployment. No live Discord API calls were used for validation.

## Original validation (before SQLite)

| Check | Result |
| --- | --- |
| `cargo check --workspace --all-targets --locked` | Passed (default dev configuration). |
| `cargo check --workspace --all-targets --no-default-features --features prod --locked` | Passed; also exposed and verified the environment-macro fix. |
| `cargo test --workspace --locked` | Passed: 22 bot tests and 2 macro tests; no failures. |
| `cargo +nightly fmt --all -- --check` | Passed with the repository's existing configuration. |
| `git diff --check` | Passed. |
| Source/asset marker search | No remaining TODO/FIXME/FILLME/unimplemented markers in `src`, `macros`, `common`, or `assets`. |

The Windows linker still emits its import-library creation warning for the procedural macro DLL. No Rust code warnings or the previous future-incompatibility warning remain in these checks. Docker images and live Discord behavior were not exercised.
