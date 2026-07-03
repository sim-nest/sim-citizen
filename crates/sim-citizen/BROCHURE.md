# sim-citizen

In one line: The layer that lets a domain's own data types show up as first-class, well-behaved values inside SIM.

## What it gives you

A citizen is any value that SIM treats as a proper public thing: it can be read in from text, written back out, checked for correct behavior, and listed in a running system's inventory. This crate holds the shared support that makes a type into a citizen. It records each type in a registry, installs it into a running library and context, runs a conformance check that reads a value in and confirms it comes back out unchanged, compares two values for real meaning rather than surface bytes, and produces both a browse card and a census row so anyone can see what lives in the system. It is the practical machinery that turns a plain Rust type into something SIM can host, list, and trust.

## Why you will be glad

- Your own data types become full participants that the rest of SIM can read, print, list, and compare.
- A built-in round-trip check catches a type that reads back differently from how it was written, before it ships.
- Every hosted type shows up in a census and a browse card, so the running system stays inspectable rather than opaque.

## Where it fits

SIM keeps its kernel small and pushes concrete behavior into loadable libraries. This crate is the Rust-side conformance layer for one kernel contract: the citizen. The kernel says what a public value must offer; this crate supplies the shared support that registers a type against that contract and installs it. It sits between a domain author's types and the runtime, and its companion derive crate writes most of this support automatically. Reading a value in stays gated by the codec and runtime path, so hosting a type here does not by itself grant anyone permission to construct one.
