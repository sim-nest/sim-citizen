use sim_citizen_derive::non_citizen;

#[non_citizen(
    reason = "runtime-owned state",
    reason = "duplicate",
    kind = "live-handle",
    descriptor = "example/live-handle"
)]
struct DuplicateReason;

fn main() {}
