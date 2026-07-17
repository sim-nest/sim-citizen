include!("../support/citizen_shim.rs");

use sim_citizen_derive::Citizen;

fn __sim_citizen_install_collision_case(_linker: &mut Linker<'_>) -> Result<()> {
    Ok(())
}

fn __sim_citizen_conformance_collision_case(_cx: &mut Cx) -> Result<()> {
    Ok(())
}

#[derive(Clone, Debug, Default, PartialEq, Citizen)]
#[citizen(symbol = "example/CollisionCase", version = 1)]
struct CollisionCase {
    value: i64,
}

fn main() {
    let _ = CollisionCase::citizen_version();
}
