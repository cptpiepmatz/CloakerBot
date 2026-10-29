# Changes and decisions

Each original TODO, FIXME, and FILLME is handled separately. No remote API tests are added; validation uses compilation, formatting, and the existing local tests. Live Discord behavior still needs deployment verification.

## Completed items

3. `cloaker` countered: used the dedicated countered quote.

2. `cloaker` success: used the shared success quote instead of returning a placeholder to Discord.

1. `assets/quotes.toml`: replaced random-configuration placeholder quotes and added shared takedown/counter quotes. Quotes stay in the existing typed, compile-time TOML assets rather than being scattered through handlers.
