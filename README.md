# sim-citizen

Make a plain Rust type into a first-class SIM value -- one that reads in from
text, writes back out, checks its own round trip, and lists itself in a running
system's inventory -- usually with a single `#[derive(Citizen)]`.

SIM is a small Rust protocol kernel plus loadable libraries. This is a library,
not a runnable binary; add it to a crate and call it from Rust. For the full
run-it walkthrough of the constellation, see sim-say
(github.com/sim-nest/sim-say).

## Example

```bash
cargo add sim-citizen
```

Parse a `namespace/name` string into a kernel symbol -- the spelling used for
the symbols recorded in citizen registry rows:

```rust
use sim_citizen::parse_symbol;

let qualified = parse_symbol("example/Point");
assert_eq!(qualified.to_string(), "example/Point");

let bare = parse_symbol("Point");
assert_eq!(bare.to_string(), "Point");
```

(from the `parse_symbol` doctest, `crates/sim-citizen/src/symbol.rs:13`)

## How it works

sim-citizen is the Rust-side conformance layer for the SIM constellation. A
citizen is a public SIM-facing runtime value with a class-backed read
constructor, constructor encoding, conformance fixture, and census row;
sim-citizen owns the shared support that registers domain types against the
kernel's citizen contract -- registry rows, runtime installation helpers,
fixture checks, generated census rendering, and the semantic equality helpers
behind the strict citizen gate.

Domain types usually opt in with `#[derive(Citizen)]`, which generates that
support from `#[citizen(...)]` attributes; hard cases register hand-written
citizens, and live handles carry inline `#[non_citizen]` exemptions that name
their descriptor strategy. Read-construct stays capability-gated by the
codec/runtime path, not by this layer.

## Crates

- `sim-citizen` -- the citizen support layer: registry rows, runtime
  installation, conformance fixtures, field and equality traits, and census and
  card rendering.
- `sim-citizen-derive` -- the proc-macro crate providing `#[derive(Citizen)]`
  and the `#[non_citizen]` exemption attribute, targeting the sim-citizen
  support layer.

## Validation

These commands run in the constellation workspace; only `sim-kernel` builds from a lone clone today (see `DEVELOPING.md` in `sim-sdk`). A single-repo build lands with the first crates.io publish.

```bash
cargo fmt --check && cargo test --workspace && cargo clippy --workspace -- -D warnings && cargo doc --workspace --no-deps
cargo run -p xtask -- simdoc --check
```

## Documentation Lanes

`cargo run -p xtask -- simdoc` builds the public documentation lanes:

- API docs: `target/doc/`
- Agent cards: `docs/agents/cards.jsonl` and `docs/agents/card-index.json`
- Human docs: `docs/humans/`
- Diagrams: `docs/diagrams/src/` and `docs/diagrams/generated/`

The same command writes split contract files under `docs/generated/`. Everything
under `docs/` is generated; do not hand-edit it.

### Rustdoc conventions

Public API documentation in `src/` follows one house style:

- Every public item opens with a one-line summary sentence, then context.
- A type that satisfies a kernel contract states which contract it implements:
  the kernel defines the contract; sim-citizen is the Rust-side conformance layer
  that registers types against it.
- The first-reach types carry a `# Examples` doctest that compiles and passes.
- Cross-reference with intra-doc links, and link back to this README rather than
  restating it.

The public API is documentation-gated: each crate's `lib.rs` denies
`missing_docs`, so every public item, field, and macro must be documented for the
crate to build.

### Examples and recipes

The crates' examples are their rustdoc doctests, the `example` reference citizen,
and the in-crate conformance fixtures. Neither crate ships a `recipes/` tree: a
runnable recipe that exercises a citizen end to end needs a codec and a runtime
to read-construct and evaluate it, which this support layer does not load.
Recipes that register and drive citizens live in the crates that load those.
