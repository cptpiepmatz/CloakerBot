# Runtime logging

CloakerBot uses `tracing` for runtime diagnostics. Discord-channel takedown
notifications remain separate and are configured through the bot's commands.

Logs go to stderr. Text output includes local timestamps, levels, module targets,
and operation spans; colors are enabled only when stderr is a terminal. Set
`LOG_FORMAT=json` for newline-delimited JSON with structured fields and span context.
The default is `LOG_FORMAT=text`.

Both formats use the system's local timezone, with the UTC offset included in
the timestamp. Containers use their own timezone configuration, which may default
to UTC rather than the host's timezone.

## Filtering

Use `RUST_LOG` to override the default filter:

```text
RUST_LOG=info,CloakerBot::takedown=debug,sqlx=warn
```

By default, dependencies log at warning level, while CloakerBot logs at debug
level in debug builds and info level in release builds. The application target is
`CloakerBot` (case-sensitive, matching the binary name). `trace` is also available
for deeper dependency diagnostics. Invalid filters or output formats fail startup
instead of silently falling back. The old `LOG_LEVEL` variable is no longer used.

## Context

Events, interactions, takedowns, OAuth handlers, backups, and linked-role workers
have spans. Guild, user, target, and interaction IDs are recorded as appropriate.
OAuth handler arguments are deliberately excluded: never record authorization
codes, cookies, tokens, client secrets, or full interaction payloads.

For new events, prefer named fields over embedding values in messages:

```rust
tracing::warn!(%guild_id, ?error, "Could not log takedown to Discord");
```

Use `#[tracing::instrument(skip_all, fields(%guild_id))]` on async operation
boundaries. Propagate the current span into spawned futures with
`tracing::Instrument::in_current_span()`. Do not hold a span's `enter()` guard
across `.await`. For blocking tasks, capture `tracing::Span::current()` and use
`span.in_scope(...)` inside the blocking closure.

`miette` is reserved for rich error reports when an error terminates the application.
Recoverable runtime failures retain their typed error enums and are logged directly
through tracing, alongside the operation context. Use `?error` to retain nested enum
details rather than embedding a fancy terminal report in a log field.
No external telemetry service is required.

## Panic reports

`src/panic.rs` installs the application panic hook at the beginning of `main`.
It renders `miette::Report` panic payloads directly, retaining their error chains,
and falls back to miette's regular panic hook for other payloads.

Use `ExpectReport::expect_report` at explicit panic boundaries such as lazy startup
initialization:

```rust
use crate::panic::ExpectReport;

let value = result.expect_report("could not init resource");
```

The extension supports both `Result` and `Option`. For `Result`, the original
typed error remains the source; for `Option`, the supplied message is the
diagnostic. `#[track_caller]` preserves the call site's file, line, and column
through the extension and panic helper. `STATE` remains a `LazyLock`.
The panic diagnostic identifies the panicking thread as the top-level error,
retains the original report as its standard error source, forwards its diagnostic
metadata, and adds the caller location as help text. It does not nest the original
report as a diagnostic source, avoiding a second graphical error marker. Miette
formats the complete report without separate headers or an added `Error:` prefix.

This is still a panic, not error recovery or a forced process exit. In a spawned
Tokio task it can become a `JoinError`; existing unwinding and task behavior are
unchanged. Recoverable failures should continue to use typed `Result`s and tracing,
not `expect_report`.
