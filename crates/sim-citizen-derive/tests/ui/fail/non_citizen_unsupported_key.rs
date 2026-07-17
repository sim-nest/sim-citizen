use sim_citizen_derive::non_citizen;

#[non_citizen(
    reason = "runtime-owned state",
    kind = "live-handle",
    descriptor = "example/live-handle",
    extra = "unexpected"
)]
struct UnsupportedKey;

fn main() {}
