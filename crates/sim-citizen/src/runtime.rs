//! Runtime installation helpers for registering citizens with a context.

use std::{
    fmt::Debug,
    marker::PhantomData,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};

use sim_kernel::{
    Args, Callable, Class, ClassId, ClassRef, Cx, DefaultFactory, Expr, Factory, Object,
    ObjectEncode, ObjectEncoding, ReadConstructor, ReadConstructorRef, Result, ShapeRef, Symbol,
    TableRef, Value,
};

use crate::{CitizenInfo, arity_error, parse_symbol};

/// Static citizen identity: the metadata `#[derive(Citizen)]` supplies.
///
/// Reports the citizen's class symbol, encoding version, constructor arity, and
/// field names. The kernel owns `Symbol`; this trait is the Rust-side identity
/// a citizen registers against the kernel's class and read-construct contracts.
///
/// # Examples
///
/// `#[derive(Citizen)]` supplies this identity from the `#[citizen(...)]`
/// attribute and the struct's fields:
///
/// ```
/// # use sim_citizen::Citizen;
/// # use sim_citizen_derive::Citizen;
/// #[derive(Clone, Debug, Default, PartialEq, Citizen)]
/// #[citizen(symbol = "doc/Point", version = 1)]
/// struct Point {
///     x: i64,
///     y: i64,
/// }
///
/// assert_eq!(Point::citizen_symbol().to_string(), "doc/Point");
/// assert_eq!(Point::citizen_version(), 1);
/// assert_eq!(Point::citizen_arity(), 2);
/// assert_eq!(Point::citizen_fields(), &["x", "y"]);
/// ```
pub trait Citizen: Clone + Send + Sync + 'static {
    /// The citizen's `namespace/name` class symbol.
    fn citizen_symbol() -> Symbol;
    /// The citizen's encoding version.
    fn citizen_version() -> u32;
    /// Number of constructor fields (excluding the version argument).
    fn citizen_arity() -> usize;
    /// The citizen's field names, in constructor order.
    fn citizen_fields() -> &'static [&'static str];
}

/// A fully runnable citizen: identity plus kernel object and read-construct.
///
/// Combines [`Citizen`] with the kernel object, encoding, and equality bounds
/// needed to install the type as a class and round-trip it through
/// read-construct. Read-construct stays capability-gated by the runtime path,
/// not by implementing this trait.
pub trait CitizenRuntime:
    Citizen + Object + sim_kernel::ObjectCompat + ObjectEncode + PartialEq + Debug
{
    /// Returns the static registry row for this citizen type.
    ///
    /// The derive emits this metadata independently of the inventory row, so a
    /// crate can register its citizens explicitly with
    /// [`CitizenRegistry::register`](crate::CitizenRegistry::register) in
    /// builds where link-time constructor collection is unsuitable.
    fn citizen_info() -> CitizenInfo;
    /// Installs this citizen's class into a kernel linker.
    ///
    /// The default implementation backs the generated registry row without
    /// reserving an inherent helper name on the user's type.
    fn install(linker: &mut sim_kernel::Linker<'_>) -> Result<()>
    where
        Self: Sized,
    {
        install_derived::<Self>(linker)
    }
    /// Runs this citizen's conformance fixture set.
    fn conformance(cx: &mut Cx) -> Result<()>;
    /// Builds the citizen from decoded constructor argument values.
    fn construct_from_values(cx: &mut Cx, args: Vec<Value>) -> Result<Self>;
    /// Returns the canonical example value used as a conformance fixture.
    fn example() -> Self;
}

/// Installs a derived citizen `T` as a class value in `linker`.
///
/// Registers a class backing `T`'s callable, class, and read-constructor
/// contracts and binds it under `T::citizen_symbol`. This is the call a
/// citizen's generated [`InstallFn`](crate::InstallFn) makes.
pub fn install_derived<T>(linker: &mut sim_kernel::Linker<'_>) -> Result<()>
where
    T: CitizenRuntime,
{
    let class = Arc::new(DerivedCitizenClass::<T>::new());
    let id = linker.class_value(
        T::citizen_symbol(),
        DefaultFactory
            .opaque(class.clone())
            .expect("citizen class should be boxable"),
    )?;
    class.set_id(id);
    Ok(())
}

/// Encodes a citizen value as its read-construct `Expr` extension form.
///
/// Maps the value's [`ObjectEncoding`] to the matching `Expr::Extension`: a
/// constructor encoding becomes a `citizen/read-construct` vector, tagged data
/// becomes a tagged map, and an opaque encoding becomes a tagged stable id.
pub fn constructor_expr<T>(cx: &mut Cx, value: &T) -> Result<Expr>
where
    T: Citizen + ObjectEncode,
{
    match value.object_encoding(cx)? {
        ObjectEncoding::Constructor { class, args } => Ok(Expr::Extension {
            tag: Symbol::qualified("citizen", "read-construct"),
            payload: Box::new(Expr::Vector(
                std::iter::once(Expr::Symbol(class)).chain(args).collect(),
            )),
        }),
        ObjectEncoding::TaggedData { tag, fields } => Ok(Expr::Extension {
            tag,
            payload: Box::new(Expr::Map(
                fields
                    .into_iter()
                    .map(|(key, value)| (Expr::Symbol(key), value))
                    .collect(),
            )),
        }),
        ObjectEncoding::Opaque { class, stable_id } => Ok(Expr::Extension {
            tag: class,
            payload: Box::new(Expr::String(stable_id)),
        }),
    }
}

pub struct DerivedCitizenClass<T> {
    id: AtomicU32,
    marker: PhantomData<T>,
}

impl<T> DerivedCitizenClass<T> {
    fn new() -> Self {
        Self {
            id: AtomicU32::new(0),
            marker: PhantomData,
        }
    }

    fn set_id(&self, id: ClassId) {
        self.id.store(id.0, Ordering::Relaxed);
    }
}

impl<T> Object for DerivedCitizenClass<T>
where
    T: CitizenRuntime,
{
    fn display(&self, _cx: &mut Cx) -> Result<String> {
        Ok(format!("#<class {}>", T::citizen_symbol()))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl<T> sim_kernel::ObjectCompat for DerivedCitizenClass<T>
where
    T: CitizenRuntime,
{
    fn class(&self, cx: &mut Cx) -> Result<ClassRef> {
        if let Some(value) = cx
            .registry()
            .class_by_symbol(&Symbol::qualified("core", "Class"))
        {
            return Ok(value.clone());
        }
        cx.factory().class_stub(
            sim_kernel::CORE_CLASS_CLASS_ID,
            Symbol::qualified("core", "Class"),
        )
    }

    fn as_expr(&self, _cx: &mut Cx) -> Result<Expr> {
        Ok(Expr::Symbol(T::citizen_symbol()))
    }

    fn as_callable(&self) -> Option<&dyn Callable> {
        Some(self)
    }

    fn as_class(&self) -> Option<&dyn Class> {
        Some(self)
    }

    fn as_read_constructor(&self) -> Option<&dyn ReadConstructor> {
        Some(self)
    }
}

impl<T> Callable for DerivedCitizenClass<T>
where
    T: CitizenRuntime,
{
    fn call(&self, cx: &mut Cx, args: Args) -> Result<Value> {
        let value = T::construct_from_values(cx, args.into_vec())?;
        cx.factory().opaque(Arc::new(value))
    }
}

impl<T> Class for DerivedCitizenClass<T>
where
    T: CitizenRuntime,
{
    fn id(&self) -> ClassId {
        ClassId(self.id.load(Ordering::Relaxed))
    }

    fn symbol(&self) -> Symbol {
        T::citizen_symbol()
    }

    fn constructor_shape(&self, cx: &mut Cx) -> Result<ShapeRef> {
        any_shape(cx)
    }

    fn instance_shape(&self, cx: &mut Cx) -> Result<ShapeRef> {
        any_shape(cx)
    }

    fn read_constructor(&self, cx: &mut Cx) -> Result<Option<ReadConstructorRef>> {
        Ok(cx.registry().class_by_symbol(&T::citizen_symbol()).cloned())
    }

    fn members(&self, cx: &mut Cx) -> Result<TableRef> {
        citizen_metadata_table(
            cx,
            T::citizen_version(),
            T::citizen_arity(),
            T::citizen_fields(),
        )
    }
}

impl<T> ReadConstructor for DerivedCitizenClass<T>
where
    T: CitizenRuntime,
{
    fn symbol(&self) -> Symbol {
        T::citizen_symbol()
    }

    fn args_shape(&self, cx: &mut Cx) -> Result<ShapeRef> {
        any_shape(cx)
    }

    fn construct_read(&self, cx: &mut Cx, args: Vec<Value>) -> Result<Value> {
        if args.len() != T::citizen_arity() + 1 {
            return Err(arity_error(
                T::citizen_symbol(),
                T::citizen_arity() + 1,
                args.len(),
            ));
        }
        self.call(cx, Args::new(args))
    }
}

fn any_shape(cx: &mut Cx) -> Result<ShapeRef> {
    if let Some(shape) = cx
        .registry()
        .shape_by_symbol(&Symbol::qualified("core", "Any"))
    {
        Ok(shape.clone())
    } else {
        cx.factory().nil()
    }
}

fn citizen_metadata_table(
    cx: &mut Cx,
    version: u32,
    arity: usize,
    fields: &[&str],
) -> Result<TableRef> {
    let fields = fields
        .iter()
        .map(|field| cx.factory().symbol(Symbol::new((*field).to_owned())))
        .collect::<Result<Vec<_>>>()?;
    cx.factory().table(vec![
        (
            Symbol::new("version"),
            cx.factory()
                .number_literal(parse_symbol("citizen/int"), version.to_string())?,
        ),
        (
            Symbol::new("arity"),
            cx.factory()
                .number_literal(parse_symbol("citizen/int"), arity.to_string())?,
        ),
        (Symbol::new("fields"), cx.factory().list(fields)?),
    ])
}
