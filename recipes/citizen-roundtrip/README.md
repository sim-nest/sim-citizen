# Citizen Roundtrip Recipe

This runnable recipe derives a `Widget` citizen, registers it through an
explicit `CitizenRegistry`, runs the expected-symbol conformance sweep, and
prints the generated citizen census.

Run it from the repository root:

```bash
cargo run --manifest-path recipes/citizen-roundtrip/Cargo.toml
```

The output includes the recipe citizen row and the Markdown census table that a
host can expose through browse or documentation lanes.
