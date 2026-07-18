# sim-citizen-derive

`sim-citizen-derive` provides `#[derive(Citizen)]` for Rust domain types that
need the sim-citizen registry, read-constructor, conformance, and census support.
It also provides `#[non_citizen]` for explicit, descriptor-named opt-outs.

## Install

```bash
cargo add sim-citizen sim-citizen-derive
```

## Derive Shape

```rust
use sim_citizen_derive::Citizen;

#[derive(Clone, Debug, Default, PartialEq, Citizen)]
#[citizen(symbol = "example/Point", version = 1)]
struct Point {
    x: i64,
    y: i64,
}
```

The derive accepts this attribute grammar:

- Type-level `#[citizen(...)]` keys `symbol = "namespace/Name"` and
  `version = N` are required.
- `example = path::to::fixture_fn` names a zero-argument function returning the
  canonical `Self` fixture. Without it, the derive calls `Default::default()`.
- `fixtures = path::to::fixtures_fn` names a zero-argument function whose return
  value implements `IntoIterator<Item = Self>`. Every emitted fixture runs
  through conformance.
- Field-level `#[citizen(with = path::to::codec)]` names a module with
  `encode(&FieldTy) -> sim_kernel::Expr` and
  `decode(&sim_kernel::Expr) -> sim_kernel::Result<FieldTy>`.
- Field markers `#[citizen(list)]` and `#[citizen(citizen)]` are rejected. Use
  `Vec<T>`, `Option<T>`, or an explicit `with` codec.

## Non-Citizen Shape

```rust
use sim_citizen_derive::non_citizen;

#[non_citizen(
    reason = "runtime-owned state",
    kind = "live-handle",
    descriptor = "example/live-handle"
)]
struct LiveHandle;
```

The opt-out marker records the type name, crate name, reason, kind, and
descriptor in the sim-citizen inventory, so host-owned handles are explicit
rather than silent gaps.

## Links

The repository root README explains the crate group. The
`recipes/citizen-roundtrip` recipe shows the derive in a runnable context. API
documentation is on docs.rs under `sim-citizen-derive`.
