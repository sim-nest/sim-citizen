use std::sync::Arc;

use sim_citizen::{
    CitizenRegistry, citizen_registry_census_markdown, run_registry_conformance_expecting,
};
use sim_citizen_derive::Citizen;
use sim_kernel::{Cx, DefaultFactory, Error, NoopEvalPolicy};

#[derive(Clone, Debug, PartialEq, Citizen)]
#[citizen(symbol = "recipe/Widget", version = 1, example = widget_example)]
struct Widget {
    id: i64,
    name: String,
}

fn widget_example() -> Widget {
    Widget {
        id: 7,
        name: "field badge".to_owned(),
    }
}

fn main() -> sim_kernel::Result<()> {
    let mut cx = Cx::new(Arc::new(NoopEvalPolicy), Arc::new(DefaultFactory));
    let mut registry = CitizenRegistry::new();
    registry.register::<Widget>()?;

    // The explicit registry path names the expected citizen type before it
    // exercises the read-construct round trip.
    run_registry_conformance_expecting(&mut cx, &registry, &["recipe/Widget"])?;

    let widget_symbol = <Widget as sim_citizen::Citizen>::citizen_symbol();
    if cx.registry().class_by_symbol(&widget_symbol).is_none() {
        return Err(Error::Eval(format!(
            "recipe citizen {widget_symbol} was not installed"
        )));
    }

    let row = registry
        .citizens()
        .find(|info| info.symbol == "recipe/Widget")
        .ok_or_else(|| Error::Eval("recipe/Widget was not registered".to_owned()))?;
    println!(
        "registered citizen: {} v{} ({} fields) from {}",
        row.symbol, row.version, row.arity, row.crate_name
    );
    println!("{}", citizen_registry_census_markdown(&registry));
    Ok(())
}
