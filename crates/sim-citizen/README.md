# sim-citizen

`sim-citizen` is the Rust support layer for SIM citizen values: domain objects
that have a public class symbol, read-constructor shape, conformance fixture,
semantic equality check, browse card, and census row.

## Install

```bash
cargo add sim-citizen
```

Most Rust domain types pair this crate with `sim-citizen-derive`:

```bash
cargo add sim-citizen-derive
```

## What It Provides

- `Citizen` and `CitizenRuntime` describe the class symbol, version, fields,
  constructor encoding, fixture, and runtime object hooks for a domain type.
- `CitizenLib` installs registered citizens into a `sim_kernel::Cx` so the
  class-backed read constructor and browse surfaces are available.
- `run_registered_conformance` executes every registered fixture through the
  read-construct round-trip gate.
- `CitizenField` encodes and decodes supported scalar, list, option, and custom
  field values.
- `citizen_card` and `citizen_census_markdown` expose the inventory that a host
  or reviewer can inspect.

## Contract Shape

A citizen publishes a `namespace/Name` class symbol, a numeric version, and a
fixed field order. Constructor encoding writes a tagged SIM expression with a
version argument followed by the field values. Decoding checks arity, version,
field domains, and semantic equality. Read-construction remains gated by the
runtime and codec path; this crate supplies the contract support, not ambient
construction permission.

## Quick Use

```rust
use sim_citizen::{CitizenLib, run_registered_conformance};
use sim_kernel::{Cx, DefaultFactory, NoopEvalPolicy};
use std::sync::Arc;

fn install_and_check() -> sim_kernel::Result<()> {
    let mut cx = Cx::new(Arc::new(NoopEvalPolicy), Arc::new(DefaultFactory));
    cx.load_lib(&CitizenLib::all())?;
    run_registered_conformance(&mut cx)
}
```

The repository root README explains the crate group. The
`recipes/citizen-roundtrip` recipe shows a complete derived citizen that
registers, runs conformance, and prints its census row. API documentation is on
docs.rs under `sim-citizen`.
