use std::sync::Arc;

use sim_citizen::{citizen_census_markdown, registered_citizens, run_registered_conformance};
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

    // The conformance sweep loads CitizenLib::all(), installing every linked
    // inventory row before it exercises the read-construct round trip.
    run_registered_conformance(&mut cx)?;

    let widget_symbol = <Widget as sim_citizen::Citizen>::citizen_symbol();
    if cx.registry().class_by_symbol(&widget_symbol).is_none() {
        return Err(Error::Eval(format!(
            "recipe citizen {widget_symbol} was not installed"
        )));
    }

    let row = registered_citizens()
        .find(|info| info.symbol == "recipe/Widget")
        .ok_or_else(|| Error::Eval("recipe/Widget was not registered".to_owned()))?;
    println!(
        "registered citizen: {} v{} ({} fields) from {}",
        row.symbol, row.version, row.arity, row.crate_name
    );
    println!("{}", citizen_census_markdown());
    Ok(())
}
