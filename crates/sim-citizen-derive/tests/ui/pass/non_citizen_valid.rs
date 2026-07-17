extern crate self as sim_citizen;

pub use ::inventory;
use sim_citizen_derive::non_citizen;

pub struct NonCitizenInfo {
    pub type_name: &'static str,
    pub crate_name: &'static str,
    pub reason: &'static str,
    pub kind: &'static str,
    pub descriptor: &'static str,
}

inventory::collect!(NonCitizenInfo);

#[non_citizen(
    reason = "runtime-owned state",
    kind = "live-handle",
    descriptor = "example/live-handle"
)]
struct ExampleLiveHandle;

fn main() {
    let _ = core::mem::size_of::<ExampleLiveHandle>();
}
