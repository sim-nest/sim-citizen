//! Builds the read-construct `Expr` for a versioned text citizen form.

use sim_kernel::{Expr, Symbol};

/// Builds the `citizen/read-construct` [`Expr`] for a `v1` text citizen form.
///
/// Text-form citizens (pitch, chord, MIDI, and sound shapes) read-construct
/// from a single canonical string tagged with the `v1` form version. This is
/// the shared spelling of that wire form: a `citizen/read-construct` extension
/// whose payload is the vector `[class, v1, form]`.
///
/// The kernel owns `Expr` and `Symbol`; this helper only assembles them, so it
/// adds no dependency beyond the kernel.
///
/// # Examples
///
/// ```
/// # use sim_citizen::text_read_construct_expr;
/// # use sim_kernel::{Expr, Symbol};
/// let class = Symbol::qualified("pitch", "Pitch");
/// let expr = text_read_construct_expr(class.clone(), "C4");
/// assert_eq!(
///     expr,
///     Expr::Extension {
///         tag: Symbol::qualified("citizen", "read-construct"),
///         payload: Box::new(Expr::Vector(vec![
///             Expr::Symbol(class),
///             Expr::Symbol(Symbol::new("v1")),
///             Expr::String("C4".to_owned()),
///         ])),
///     }
/// );
/// ```
pub fn text_read_construct_expr(class: Symbol, form: impl Into<String>) -> Expr {
    Expr::Extension {
        tag: Symbol::qualified("citizen", "read-construct"),
        payload: Box::new(Expr::Vector(vec![
            Expr::Symbol(class),
            Expr::Symbol(Symbol::new("v1")),
            Expr::String(form.into()),
        ])),
    }
}

#[cfg(test)]
mod tests {
    use super::text_read_construct_expr;
    use sim_kernel::{Expr, Symbol};

    /// Reproduces the wire form the music shape crates hand-roll and asserts the
    /// helper matches it: class symbol, `v1` tag, string payload, in order.
    fn shape_crate_read_construct_expr(class: Symbol, form: String) -> Expr {
        Expr::Extension {
            tag: Symbol::qualified("citizen", "read-construct"),
            payload: Box::new(Expr::Vector(vec![
                Expr::Symbol(class),
                Expr::Symbol(Symbol::new("v1")),
                Expr::String(form),
            ])),
        }
    }

    #[test]
    fn text_read_construct_expr_matches_shape_crate_form() {
        let class = Symbol::qualified("pitch", "Chord");
        let expr = text_read_construct_expr(class.clone(), "C4,E4,G4");
        assert_eq!(
            expr,
            shape_crate_read_construct_expr(class, "C4,E4,G4".to_owned())
        );
    }

    #[test]
    fn text_read_construct_expr_has_class_v1_and_string_payload() {
        let class = Symbol::qualified("pitch", "Pitch");
        let expr = text_read_construct_expr(class.clone(), "C4");
        match expr {
            Expr::Extension { tag, payload } => {
                assert_eq!(tag, Symbol::qualified("citizen", "read-construct"));
                match *payload {
                    Expr::Vector(items) => {
                        assert_eq!(
                            items,
                            vec![
                                Expr::Symbol(class),
                                Expr::Symbol(Symbol::new("v1")),
                                Expr::String("C4".to_owned()),
                            ]
                        );
                    }
                    other => panic!("payload must be a vector, found {other:?}"),
                }
            }
            other => panic!("must be an extension, found {other:?}"),
        }
    }
}
