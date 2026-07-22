use sim_citizen_derive::non_citizen;

#[non_citizen(reason = "runtime-owned state", descriptor = "example/live-handle")]
struct MissingKind;

fn main() {}
