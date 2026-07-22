# sim-citizen-derive

In one line: A one-line marker that writes all the wiring needed to make your data type a full SIM value for you.

## What it gives you

Making a type into a proper SIM value takes a fair amount of repetitive support: a registry entry, install steps, a conformance check, field handling, and equality comparison. This crate lets you skip writing that by hand. You place a short marker above your type and describe it with a few plain attributes -- its public name, its version, and how its fields behave -- and the marker generates the matching support for you. It also offers a second marker that stamps a type as a deliberate, named exception when it should not be a hosted value at all, so the exemption is explicit and on the record rather than a silent gap.

## Why you will be glad

- One short marker replaces a page of hand-written wiring, so adding a hosted type stays quick and hard to get wrong.
- The generated support always matches the current contract, so your types and the runtime stay in step as things change.
- An explicit opt-out marker records every deliberate exception by name, so nothing is quietly left out without a reason.

## Where it fits

This is the companion tool to the sim-citizen support layer. Where that crate holds the shared machinery a hosted value needs, this crate is the proc-macro that produces that machinery for an ordinary domain type from a handful of attributes. Most authors reach for this marker as the everyday way in; the harder cases write their support by hand against the support crate directly. It generates code for that layer, so the two crates pair naturally, with this one carrying the load for the common path.
