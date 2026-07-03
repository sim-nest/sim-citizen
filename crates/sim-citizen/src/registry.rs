//! Registry rows and library installation for registered citizens.

use sim_kernel::{
    AbiVersion, Export, Lib, LibManifest, LibTarget, Linker, LoadCx, Result, Symbol, Version,
};

use crate::parse_symbol;

/// Installs a citizen's class into a kernel [`Linker`].
pub type InstallFn = for<'a> fn(&mut Linker<'a>) -> Result<()>;
/// Runs a citizen's conformance fixture against a kernel context.
pub type ConformanceFn = fn(&mut sim_kernel::Cx) -> Result<()>;

/// One inventory-registered citizen's static registry row.
///
/// Collected via `inventory` so every linked crate's citizens are discoverable
/// without a central list. Carries identity metadata plus the install and
/// conformance hooks driving registration and the strict gate.
#[derive(Clone, Copy)]
pub struct CitizenInfo {
    /// The citizen's `namespace/name` symbol text.
    pub symbol: &'static str,
    /// The citizen's encoding version.
    pub version: u32,
    /// The crate that defined the citizen.
    pub crate_name: &'static str,
    /// Number of constructor fields (excluding the version argument).
    pub arity: usize,
    /// Hook that installs the citizen's class into a linker.
    pub install: InstallFn,
    /// Hook that runs the citizen's conformance fixture.
    pub conformance: ConformanceFn,
}

inventory::collect!(CitizenInfo);

/// Iterates every [`CitizenInfo`] registered through `inventory`.
pub fn registered_citizens() -> impl Iterator<Item = &'static CitizenInfo> {
    inventory::iter::<CitizenInfo>.into_iter()
}

/// Installs every registered citizen's class into `linker`.
pub fn install_all(linker: &mut Linker<'_>) -> Result<()> {
    for info in registered_citizens() {
        (info.install)(linker)?;
    }
    Ok(())
}

/// Installs only the registered citizens whose symbol is in `namespace`.
pub fn install_namespace(linker: &mut Linker<'_>, namespace: &str) -> Result<()> {
    for info in
        registered_citizens().filter(|info| symbol_namespace(info.symbol) == Some(namespace))
    {
        (info.install)(linker)?;
    }
    Ok(())
}

/// A kernel [`Lib`] that loads registered citizens into a context.
///
/// Either loads every registered citizen or only those in a chosen namespace.
/// Its manifest exports one class per included citizen so the runtime can
/// resolve them by symbol.
#[derive(Clone, Copy, Debug, Default)]
pub struct CitizenLib {
    namespace: Option<&'static str>,
}

impl CitizenLib {
    /// Returns a lib that loads every registered citizen.
    pub fn all() -> Self {
        Self { namespace: None }
    }

    /// Returns a lib that loads only citizens in `namespace`.
    pub fn namespace(namespace: &'static str) -> Self {
        Self {
            namespace: Some(namespace),
        }
    }
}

impl Lib for CitizenLib {
    fn manifest(&self) -> LibManifest {
        let id = match self.namespace {
            Some(namespace) => Symbol::qualified("citizen", namespace.to_owned()),
            None => Symbol::qualified("citizen", "all"),
        };
        LibManifest {
            id,
            version: Version(env!("CARGO_PKG_VERSION").to_owned()),
            abi: AbiVersion { major: 0, minor: 1 },
            target: LibTarget::HostRegistered,
            requires: Vec::new(),
            capabilities: Vec::new(),
            exports: registered_citizens()
                .filter(|info| match self.namespace {
                    Some(namespace) => symbol_namespace(info.symbol) == Some(namespace),
                    None => true,
                })
                .map(|info| Export::Class {
                    symbol: parse_symbol(info.symbol),
                    class_id: None,
                })
                .collect(),
        }
    }

    fn load(&self, _cx: &mut LoadCx, linker: &mut Linker<'_>) -> Result<()> {
        match self.namespace {
            Some(namespace) => install_namespace(linker, namespace),
            None => install_all(linker),
        }
    }
}

fn symbol_namespace(symbol: &str) -> Option<&str> {
    symbol.split_once('/').map(|(namespace, _)| namespace)
}
