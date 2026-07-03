//! Parses a namespace/name string into a kernel Symbol.

use sim_kernel::Symbol;

/// Parses `namespace/name` text into a kernel [`Symbol`].
///
/// Splits on the first `/` into a qualified symbol; text with no `/` becomes a
/// bare symbol. The kernel owns `Symbol`; this is the citizen-side spelling of
/// the symbols recorded in registry rows.
///
/// # Examples
///
/// ```
/// # use sim_citizen::parse_symbol;
/// let qualified = parse_symbol("example/Point");
/// assert_eq!(qualified.to_string(), "example/Point");
///
/// let bare = parse_symbol("Point");
/// assert_eq!(bare.to_string(), "Point");
/// ```
pub fn parse_symbol(text: &str) -> Symbol {
    match text.split_once('/') {
        Some((namespace, name)) => Symbol::qualified(namespace.to_owned(), name.to_owned()),
        None => Symbol::new(text.to_owned()),
    }
}
