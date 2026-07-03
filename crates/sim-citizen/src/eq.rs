//! The CitizenEq semantic-equality helper used by the strict citizen gate.

use sim_kernel::{Cx, Expr, ObjectEncoding, Result, Value};

/// Semantic equality between citizen field values for the strict gate.
///
/// Implementations compare values the way the citizen round trip does (for
/// example, `f64` compares by canonical text), so the conformance gate accepts
/// a decoded value as equal to its original even where derived `PartialEq`
/// would be too strict or too loose.
pub trait CitizenEq<Rhs = Self> {
    /// Returns whether `self` and `rhs` are citizen-equal.
    fn citizen_eq(&self, rhs: &Rhs) -> bool;
}

macro_rules! citizen_eq_partial {
    ($($ty:ty),* $(,)?) => {
        $(impl CitizenEq for $ty {
            fn citizen_eq(&self, rhs: &Self) -> bool {
                self == rhs
            }
        })*
    };
}

citizen_eq_partial!(
    bool, String, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, usize
);

impl CitizenEq for f64 {
    fn citizen_eq(&self, rhs: &Self) -> bool {
        self.to_string() == rhs.to_string()
    }
}

impl<T> CitizenEq for Vec<T>
where
    T: CitizenEq,
{
    fn citizen_eq(&self, rhs: &Self) -> bool {
        self.len() == rhs.len()
            && self
                .iter()
                .zip(rhs.iter())
                .all(|(left, right)| left.citizen_eq(right))
    }
}

impl<T> CitizenEq for Option<T>
where
    T: CitizenEq,
{
    fn citizen_eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Some(left), Some(right)) => left.citizen_eq(right),
            (None, None) => true,
            _ => false,
        }
    }
}

/// Compares two citizen [`Value`]s for semantic equality.
///
/// When both values expose an object encoder it compares their
/// [`ObjectEncoding`]s; otherwise it falls back to comparing their `Expr`
/// projections through [`expr_citizen_eq`]. The kernel owns `Value` and the
/// encoder/`as_expr` surface; this helper applies the citizen equality rule.
pub fn values_citizen_eq(cx: &mut Cx, left: &Value, right: &Value) -> Result<bool> {
    let left_encoding = left
        .object()
        .as_object_encoder()
        .map(|encoder| encoder.object_encoding(cx))
        .transpose()?;
    let right_encoding = right
        .object()
        .as_object_encoder()
        .map(|encoder| encoder.object_encoding(cx))
        .transpose()?;

    match (left_encoding, right_encoding) {
        (Some(left), Some(right)) => Ok(object_encoding_eq(&left, &right)),
        _ => Ok(expr_citizen_eq(
            &left.object().as_expr(cx)?,
            &right.object().as_expr(cx)?,
        )),
    }
}

/// Compares two `Expr`s under citizen equality.
///
/// Equivalent to the kernel's canonical equality except that `f64`-domain
/// numbers compare by canonical text, matching how citizen fields round trip.
///
/// # Examples
///
/// ```
/// # use sim_citizen::{expr_citizen_eq, CitizenField};
/// let left = 7_i64.encode_field();
/// let right = 7_i64.encode_field();
/// assert!(expr_citizen_eq(&left, &right));
///
/// let other = 8_i64.encode_field();
/// assert!(!expr_citizen_eq(&left, &other));
/// ```
pub fn expr_citizen_eq(left: &Expr, right: &Expr) -> bool {
    match (left, right) {
        (Expr::Number(left), Expr::Number(right)) if left.domain.name.as_ref() == "f64" => {
            left.canonical == right.canonical
        }
        _ => left.canonical_eq(right),
    }
}

fn object_encoding_eq(left: &ObjectEncoding, right: &ObjectEncoding) -> bool {
    match (left, right) {
        (
            ObjectEncoding::Constructor {
                class: left_class,
                args: left_args,
            },
            ObjectEncoding::Constructor {
                class: right_class,
                args: right_args,
            },
        ) => {
            left_class == right_class
                && left_args.len() == right_args.len()
                && left_args
                    .iter()
                    .zip(right_args.iter())
                    .all(|(left, right)| expr_citizen_eq(left, right))
        }
        (
            ObjectEncoding::TaggedData {
                tag: left_tag,
                fields: left_fields,
            },
            ObjectEncoding::TaggedData {
                tag: right_tag,
                fields: right_fields,
            },
        ) => {
            left_tag == right_tag
                && left_fields.len() == right_fields.len()
                && left_fields.iter().zip(right_fields.iter()).all(
                    |((left_name, left), (right_name, right))| {
                        left_name == right_name && expr_citizen_eq(left, right)
                    },
                )
        }
        (
            ObjectEncoding::Opaque {
                class: left_class,
                stable_id: left_id,
            },
            ObjectEncoding::Opaque {
                class: right_class,
                stable_id: right_id,
            },
        ) => left_class == right_class && left_id == right_id,
        _ => false,
    }
}
