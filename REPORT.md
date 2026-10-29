# Changes and decisions

Each original TODO, FIXME, and FILLME is handled separately. No remote API tests are added; validation uses compilation, formatting, and the existing local tests. Live Discord behavior still needs deployment verification.

## Completed items

9. Mention safety: disabled mention parsing by default on the bot HTTP client. Log messages and ordinary replies can render mentions without pinging users, roles, or everyone.

8. Ranks errors: implemented `Error` with specific context and a transparent graphics-error source, matching the existing `thiserror` style.

7. Random toggle buttons: selected the existing enabled/disabled quote according to the resulting configuration.

6. `counter` miss: used the shared missed-counter quote.

5. `counter` success: used the shared successful-counter quote.

4. `cloaker` failure: used the dedicated failure quote while preserving the specific failure reason.

3. `cloaker` countered: used the dedicated countered quote.

2. `cloaker` success: used the shared success quote instead of returning a placeholder to Discord.

1. `assets/quotes.toml`: replaced random-configuration placeholder quotes and added shared takedown/counter quotes. Quotes stay in the existing typed, compile-time TOML assets rather than being scattered through handlers.
