//! The CitizenField trait for encoding and decoding citizen field values.

use std::convert::TryFrom;

use sim_kernel::{CapabilityName, Cx, Error, Expr, NumberLiteral, Result, Symbol, Value};
use sim_value::kind::expr_kind;

/// Encodes and decodes a Rust type as one citizen constructor field.
///
/// Generated and hand-written citizens use this trait to project each field to
/// an `Expr` for the constructor encoding and to recover it on read-construct.
/// The kernel owns `Expr`/`Value`/`Cx`; sim-citizen provides the field codec
/// and the standard scalar/collection implementations.
///
/// # Examples
///
/// The expr-level encode/decode pair round-trips without a context:
///
/// ```
/// # use sim_citizen::CitizenField;
/// let original: Vec<i64> = vec![1, 2, 3];
/// let expr = original.encode_field();
/// let decoded = Vec::<i64>::decode_field_expr(&expr, "numbers").unwrap();
/// assert_eq!(decoded, original);
/// ```
pub trait CitizenField: Sized {
    /// Encodes the field value as its constructor `Expr`.
    fn encode_field(&self) -> Expr;
    /// Decodes the field from a constructor `Expr`, naming `field` in errors.
    fn decode_field_expr(expr: &Expr, field: &'static str) -> Result<Self>;

    /// Decodes the field from a runtime [`Value`] via its `Expr` projection.
    ///
    /// Default implementation projects `value` to an `Expr` through `cx` and
    /// delegates to [`CitizenField::decode_field_expr`].
    fn decode_field_value(cx: &mut Cx, value: Value, field: &'static str) -> Result<Self> {
        let expr = value_to_expr(cx, value, field)?;
        Self::decode_field_expr(&expr, field)
    }
}

/// Projects a runtime [`Value`] to an `Expr`, tagging errors with `field`.
pub fn value_to_expr(cx: &mut Cx, value: Value, field: &'static str) -> Result<Expr> {
    value
        .object()
        .as_expr(cx)
        .map_err(|err| field_error(field, format!("cannot convert value to Expr: {err}")))
}

/// Builds a runtime [`Value`] from an `Expr` using `cx`'s factory.
///
/// Reconstructs literals, symbols, lists, and symbol-keyed maps, falling back
/// to the factory's generic `expr` path for other shapes. Used to turn citizen
/// constructor arguments back into values for read-construct.
pub fn value_from_expr(cx: &mut Cx, expr: &Expr) -> Result<Value> {
    match expr {
        Expr::Nil => cx.factory().nil(),
        Expr::Bool(value) => cx.factory().bool(*value),
        Expr::Number(value) => cx
            .factory()
            .number_literal(value.domain.clone(), value.canonical.clone()),
        Expr::Symbol(value) => cx.factory().symbol(value.clone()),
        Expr::String(value) => cx.factory().string(value.clone()),
        Expr::Bytes(value) => cx.factory().bytes(value.clone()),
        Expr::List(items) => {
            let values = items
                .iter()
                .map(|item| value_from_expr(cx, item))
                .collect::<Result<Vec<_>>>()?;
            cx.factory().list(values)
        }
        Expr::Map(entries) => {
            let entries = entries
                .iter()
                .map(|(key, value)| {
                    let Expr::Symbol(key) = key else {
                        return Err(field_error(
                            "map",
                            format!("expected symbol key, found {}", expr_kind(key)),
                        ));
                    };
                    Ok((key.clone(), value_from_expr(cx, value)?))
                })
                .collect::<Result<Vec<_>>>()?;
            cx.factory().table(entries)
        }
        other => cx.factory().expr(other.clone()),
    }
}

/// Checks that a citizen's version argument equals the `expected` version.
///
/// Citizen constructors carry a leading `v<N>` version symbol; this verifies it
/// matches, returning an error naming `class` otherwise.
pub fn decode_version(cx: &mut Cx, value: Value, expected: u32, class: Symbol) -> Result<()> {
    let expected_symbol = Symbol::new(format!("v{expected}"));
    match value_to_expr(cx, value, "version")? {
        Expr::Symbol(actual) if actual == expected_symbol => Ok(()),
        other => Err(Error::Eval(format!(
            "citizen {class} expects version {expected_symbol}, found {}",
            expr_kind(&other)
        ))),
    }
}

/// Builds the standard error for a wrong read-constructor argument count.
pub fn arity_error(class: Symbol, expected: usize, actual: usize) -> Error {
    Error::Eval(format!(
        "citizen {class} expects {expected} read-constructor argument(s), found {actual}"
    ))
}

/// Builds a `citizen field <field>: <message>` evaluation error.
pub fn field_error(field: &'static str, message: impl Into<String>) -> Error {
    Error::Eval(format!("citizen field {field}: {}", message.into()))
}

fn expect_number_domain(
    number: &NumberLiteral,
    expected: Symbol,
    field: &'static str,
) -> Result<()> {
    if number.domain == expected {
        Ok(())
    } else {
        Err(field_error(
            field,
            format!("expected number domain {expected}, found {}", number.domain),
        ))
    }
}

macro_rules! signed_int_field {
    ($($ty:ty),* $(,)?) => {
        $(
            impl CitizenField for $ty {
                fn encode_field(&self) -> Expr {
                    int_expr(self.to_string())
                }

                fn decode_field_expr(expr: &Expr, field: &'static str) -> Result<Self> {
                    let value = decode_integer_text(expr, field)?;
                    <$ty>::try_from(value).map_err(|_| {
                        field_error(field, format!("integer {value} is out of range"))
                    })
                }
            }
        )*
    };
}

macro_rules! unsigned_int_field {
    ($($ty:ty),* $(,)?) => {
        $(
            impl CitizenField for $ty {
                fn encode_field(&self) -> Expr {
                    int_expr(self.to_string())
                }

                fn decode_field_expr(expr: &Expr, field: &'static str) -> Result<Self> {
                    let value = decode_integer_text(expr, field)?;
                    <$ty>::try_from(value).map_err(|_| {
                        field_error(field, format!("integer {value} is out of range"))
                    })
                }
            }
        )*
    };
}

signed_int_field!(i8, i16, i32, i64, i128, isize);
unsigned_int_field!(u8, u16, u32, u64, usize);

impl CitizenField for bool {
    fn encode_field(&self) -> Expr {
        Expr::Bool(*self)
    }

    fn decode_field_expr(expr: &Expr, field: &'static str) -> Result<Self> {
        match expr {
            Expr::Bool(value) => Ok(*value),
            other => Err(field_error(
                field,
                format!("expected bool, found {}", expr_kind(other)),
            )),
        }
    }
}

impl CitizenField for String {
    fn encode_field(&self) -> Expr {
        Expr::String(self.clone())
    }

    fn decode_field_expr(expr: &Expr, field: &'static str) -> Result<Self> {
        match expr {
            Expr::String(value) => Ok(value.clone()),
            other => Err(field_error(
                field,
                format!("expected string, found {}", expr_kind(other)),
            )),
        }
    }
}

impl CitizenField for Symbol {
    fn encode_field(&self) -> Expr {
        Expr::Symbol(self.clone())
    }

    fn decode_field_expr(expr: &Expr, field: &'static str) -> Result<Self> {
        match expr {
            Expr::Symbol(value) => Ok(value.clone()),
            other => Err(field_error(
                field,
                format!("expected symbol, found {}", expr_kind(other)),
            )),
        }
    }
}

impl CitizenField for Expr {
    fn encode_field(&self) -> Expr {
        self.clone()
    }

    fn decode_field_expr(expr: &Expr, _field: &'static str) -> Result<Self> {
        Ok(expr.clone())
    }
}

impl CitizenField for CapabilityName {
    fn encode_field(&self) -> Expr {
        Expr::String(self.as_str().to_owned())
    }

    fn decode_field_expr(expr: &Expr, field: &'static str) -> Result<Self> {
        match expr {
            Expr::String(value) => Ok(CapabilityName::new(value.clone())),
            other => Err(field_error(
                field,
                format!(
                    "expected string capability name, found {}",
                    expr_kind(other)
                ),
            )),
        }
    }
}

impl CitizenField for f64 {
    fn encode_field(&self) -> Expr {
        Expr::Number(NumberLiteral {
            domain: Symbol::qualified("numbers", "f64"),
            canonical: canonical_f64(*self),
        })
    }

    fn decode_field_expr(expr: &Expr, field: &'static str) -> Result<Self> {
        match expr {
            Expr::Number(number) => {
                expect_number_domain(number, Symbol::qualified("numbers", "f64"), field)?;
                number
                    .canonical
                    .parse::<f64>()
                    .map_err(|err| field_error(field, format!("invalid f64: {err}")))
            }
            other => Err(field_error(
                field,
                format!("expected number, found {}", expr_kind(other)),
            )),
        }
    }
}

impl<A, B> CitizenField for (A, B)
where
    A: CitizenField,
    B: CitizenField,
{
    fn encode_field(&self) -> Expr {
        Expr::List(vec![self.0.encode_field(), self.1.encode_field()])
    }

    fn decode_field_expr(expr: &Expr, field: &'static str) -> Result<Self> {
        let Expr::List(items) = expr else {
            return Err(field_error(
                field,
                format!("expected pair list, found {}", expr_kind(expr)),
            ));
        };
        let [first, second] = items.as_slice() else {
            return Err(field_error(
                field,
                format!("expected pair list with 2 item(s), found {}", items.len()),
            ));
        };
        Ok((
            A::decode_field_expr(first, field)?,
            B::decode_field_expr(second, field)?,
        ))
    }
}

impl<T> CitizenField for Vec<T>
where
    T: CitizenField,
{
    fn encode_field(&self) -> Expr {
        Expr::List(self.iter().map(CitizenField::encode_field).collect())
    }

    fn decode_field_expr(expr: &Expr, field: &'static str) -> Result<Self> {
        match expr {
            Expr::List(items) => items
                .iter()
                .map(|item| T::decode_field_expr(item, field))
                .collect(),
            other => Err(field_error(
                field,
                format!("expected list, found {}", expr_kind(other)),
            )),
        }
    }
}

impl<T> CitizenField for Option<T>
where
    T: CitizenField,
{
    fn encode_field(&self) -> Expr {
        self.as_ref()
            .map(CitizenField::encode_field)
            .unwrap_or(Expr::Nil)
    }

    fn decode_field_expr(expr: &Expr, field: &'static str) -> Result<Self> {
        match expr {
            Expr::Nil => Ok(None),
            other => T::decode_field_expr(other, field).map(Some),
        }
    }
}

fn int_expr(canonical: String) -> Expr {
    Expr::Number(NumberLiteral {
        domain: Symbol::qualified("citizen", "int"),
        canonical,
    })
}

fn decode_integer_text(expr: &Expr, field: &'static str) -> Result<i128> {
    match expr {
        Expr::Number(number) => {
            expect_number_domain(number, Symbol::qualified("citizen", "int"), field)?;
            number
                .canonical
                .parse::<i128>()
                .map_err(|err| field_error(field, format!("invalid integer: {err}")))
        }
        other => Err(field_error(
            field,
            format!("expected integer number, found {}", expr_kind(other)),
        )),
    }
}

fn canonical_f64(value: f64) -> String {
    if value.is_nan() {
        "NaN".to_owned()
    } else if value == f64::INFINITY {
        "inf".to_owned()
    } else if value == f64::NEG_INFINITY {
        "-inf".to_owned()
    } else {
        value.to_string()
    }
}
