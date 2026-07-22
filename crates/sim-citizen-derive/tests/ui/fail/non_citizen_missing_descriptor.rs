use sim_citizen_derive::non_citizen;

#[non_citizen(reason = "runtime-owned state", kind = "live-handle")]
struct MissingDescriptor;

fn main() {}
