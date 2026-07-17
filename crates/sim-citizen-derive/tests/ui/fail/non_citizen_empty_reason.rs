use sim_citizen_derive::non_citizen;

#[non_citizen(
    reason = "",
    kind = "live-handle",
    descriptor = "example/live-handle"
)]
struct EmptyReason;

fn main() {}
