extern crate self as sim_citizen;
extern crate self as sim_kernel;

use std::{any::Any, sync::Arc};

pub use ::inventory;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Symbol(String);

impl Symbol {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn qualified(namespace: &str, name: &str) -> Self {
        Self(format!("{namespace}/{name}"))
    }
}

impl core::fmt::Display for Symbol {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Nil,
    Symbol(Symbol),
    List(Vec<Expr>),
    String(String),
    Extension { tag: Symbol, payload: Box<Expr> },
}

#[derive(Clone, Debug, Default)]
pub struct Value;

#[derive(Clone, Debug, Default)]
pub struct ClassRef;

#[derive(Clone, Copy, Debug, Default)]
pub struct ClassId(pub u32);

#[derive(Debug, Default)]
pub struct Error;

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("shim error")
    }
}

impl std::error::Error for Error {}

#[derive(Default)]
pub struct Cx;

impl Cx {
    pub fn registry(&self) -> &'static Registry {
        &REGISTRY
    }

    pub fn factory(&self) -> &'static Factory {
        &FACTORY
    }
}

pub struct Registry;

impl Registry {
    pub fn class_by_symbol(&self, _symbol: &Symbol) -> Option<&'static ClassRef> {
        None
    }
}

static REGISTRY: Registry = Registry;

pub struct Factory;

impl Factory {
    pub fn class_stub(&self, _id: ClassId, _symbol: Symbol) -> Result<ClassRef> {
        Ok(ClassRef)
    }

    pub fn opaque<T>(&self, _value: Arc<T>) -> Result<Value> {
        Ok(Value)
    }
}

static FACTORY: Factory = Factory;

pub struct Linker<'a>(core::marker::PhantomData<&'a ()>);

pub struct Args(Vec<Value>);

impl Args {
    pub fn into_vec(self) -> Vec<Value> {
        self.0
    }
}

pub trait Object {
    fn display(&self, cx: &mut Cx) -> Result<String>;
    fn as_any(&self) -> &dyn Any;
}

pub trait ObjectCompat {
    fn class(&self, cx: &mut Cx) -> Result<ClassRef>;
    fn as_expr(&self, cx: &mut Cx) -> Result<Expr>;
    fn as_object_encoder(&self) -> Option<&dyn ObjectEncode>;
}

pub trait ObjectEncode {
    fn object_encoding(&self, cx: &mut Cx) -> Result<ObjectEncoding>;
}

pub enum ObjectEncoding {
    Constructor { class: Symbol, args: Vec<Expr> },
    TaggedData { tag: Symbol, fields: Vec<(Symbol, Expr)> },
    Opaque { class: Symbol, stable_id: String },
}

pub trait Citizen: Clone + Send + Sync + 'static {
    fn citizen_symbol() -> Symbol;
    fn citizen_version() -> u32;
    fn citizen_arity() -> usize;
    fn citizen_fields() -> &'static [&'static str];
}

pub trait CitizenRuntime: Citizen + Object + ObjectCompat + ObjectEncode + PartialEq + core::fmt::Debug {
    fn construct_from_values(cx: &mut Cx, args: Vec<Value>) -> Result<Self>;
    fn example() -> Self;
}

pub trait CitizenField: Sized {
    fn encode_field(&self) -> Expr;
    fn decode_field_value(cx: &mut Cx, value: Value, field: &'static str) -> Result<Self>;
}

impl CitizenField for i64 {
    fn encode_field(&self) -> Expr {
        Expr::String(self.to_string())
    }

    fn decode_field_value(_cx: &mut Cx, _value: Value, _field: &'static str) -> Result<Self> {
        Ok(0)
    }
}

impl CitizenField for String {
    fn encode_field(&self) -> Expr {
        Expr::String(self.clone())
    }

    fn decode_field_value(_cx: &mut Cx, _value: Value, _field: &'static str) -> Result<Self> {
        Ok(String::new())
    }
}

pub fn parse_symbol(value: &str) -> Symbol {
    Symbol::new(value)
}

pub fn value_to_expr(_cx: &mut Cx, _value: Value, _field: &'static str) -> Result<Expr> {
    Ok(Expr::Nil)
}

pub fn constructor_expr<T>(_cx: &mut Cx, _value: &T) -> Result<Expr>
where
    T: Citizen + ObjectEncode,
{
    Ok(Expr::Nil)
}

pub fn decode_version(_cx: &mut Cx, _value: Value, _expected: u32, _class: Symbol) -> Result<()> {
    Ok(())
}

pub fn arity_error(_class: Symbol, _expected: usize, _actual: usize) -> Error {
    Error
}

pub fn install_derived<T>(_linker: &mut Linker<'_>) -> Result<()>
where
    T: CitizenRuntime,
{
    Ok(())
}

pub fn check_default_fixture<T>(_cx: &mut Cx) -> Result<()>
where
    T: CitizenRuntime,
{
    Ok(())
}

pub fn check_fixture<T>(_cx: &mut Cx, _fixture: T) -> Result<()>
where
    T: CitizenRuntime,
{
    Ok(())
}

pub struct CitizenInfo {
    pub symbol: &'static str,
    pub version: u32,
    pub crate_name: &'static str,
    pub arity: usize,
    pub install: fn(&mut Linker<'_>) -> Result<()>,
    pub conformance: fn(&mut Cx) -> Result<()>,
}

inventory::collect!(CitizenInfo);
