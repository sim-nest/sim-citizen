//! Registry rows and library installation for registered citizens.

use std::collections::BTreeSet;

use sim_kernel::{
    AbiVersion, Error, Export, Lib, LibManifest, LibTarget, Linker, LoadCx, Result, Symbol, Version,
};

use crate::{CitizenRuntime, parse_symbol};

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

/// One inventory-registered explicit non-citizen exemption.
///
/// Collected via `inventory` so live handles and other deliberate exemptions
/// are recorded alongside citizens rather than only living in source comments.
#[derive(Clone, Copy)]
pub struct NonCitizenInfo {
    /// The Rust type name carrying the exemption.
    pub type_name: &'static str,
    /// The crate that defined the exemption.
    pub crate_name: &'static str,
    /// Why the type is exempt from citizen conformance.
    pub reason: &'static str,
    /// The exemption kind, for example `live-handle`.
    pub kind: &'static str,
    /// The named descriptor strategy the type follows instead.
    pub descriptor: &'static str,
}

inventory::collect!(NonCitizenInfo);

/// An explicit citizen registry built by naming the citizen types a crate owns.
///
/// Inventory remains available for ordinary host binaries, but strict gates and
/// wasm/LTO builds can construct this registry directly with
/// [`CitizenRegistry::register`]. That path references each citizen type in
/// ordinary Rust code, so registration does not depend on link-time constructor
/// retention.
#[derive(Clone, Default)]
pub struct CitizenRegistry {
    citizens: Vec<CitizenInfo>,
}

impl CitizenRegistry {
    /// Builds an empty explicit registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds an explicit registry from the current inventory rows.
    ///
    /// This is useful when a caller wants the registry APIs, such as
    /// completeness checks or census rendering, while still using inventory as
    /// the discovery source.
    pub fn from_inventory() -> Result<Self> {
        let mut registry = Self::new();
        for info in registered_citizens() {
            registry.register_info(*info)?;
        }
        Ok(registry)
    }

    /// Registers the citizen metadata generated for `T`.
    ///
    /// Calling this function is the DCE-safe path: the owning crate names every
    /// citizen type it expects to ship, and the registry is then loadable as a
    /// normal kernel [`Lib`].
    pub fn register<T>(&mut self) -> Result<&mut Self>
    where
        T: CitizenRuntime,
    {
        self.register_info(T::citizen_info())
    }

    /// Registers one citizen metadata row.
    ///
    /// Duplicate symbols are rejected so an explicit registry cannot silently
    /// shadow two distinct Rust types behind one SIM class symbol.
    pub fn register_info(&mut self, info: CitizenInfo) -> Result<&mut Self> {
        if self
            .citizens
            .iter()
            .any(|existing| existing.symbol == info.symbol)
        {
            return Err(Error::Eval(format!(
                "duplicate citizen registration for {}",
                info.symbol
            )));
        }
        self.citizens.push(info);
        Ok(self)
    }

    /// Iterates the citizen rows in this explicit registry.
    pub fn citizens(&self) -> impl Iterator<Item = &CitizenInfo> {
        self.citizens.iter()
    }

    /// Returns the number of citizen rows in this explicit registry.
    pub fn len(&self) -> usize {
        self.citizens.len()
    }

    /// Reports whether this explicit registry has no citizen rows.
    pub fn is_empty(&self) -> bool {
        self.citizens.is_empty()
    }

    /// Returns every expected symbol missing from this registry.
    pub fn missing_symbols<'a>(&self, expected: &'a [&'a str]) -> Vec<&'a str> {
        let found = self
            .citizens
            .iter()
            .map(|info| info.symbol)
            .collect::<BTreeSet<_>>();
        expected
            .iter()
            .copied()
            .filter(|symbol| !found.contains(symbol))
            .collect()
    }

    /// Fails closed unless this registry contains every expected symbol.
    pub fn ensure_contains_symbols(&self, expected: &[&str]) -> Result<()> {
        let missing = self.missing_symbols(expected);
        if missing.is_empty() {
            return Ok(());
        }
        Err(Error::HostError(format!(
            "citizen registry incomplete: {} expected, {} registered; missing {:?}",
            expected.len(),
            self.len(),
            missing
        )))
    }

    /// Installs every citizen row in this registry into `linker`.
    pub fn install_all(&self, linker: &mut Linker<'_>) -> Result<()> {
        for info in self.citizens() {
            (info.install)(linker)?;
        }
        Ok(())
    }

    /// Installs only the citizen rows whose symbols are in `namespace`.
    pub fn install_namespace(&self, linker: &mut Linker<'_>, namespace: &str) -> Result<()> {
        for info in self
            .citizens()
            .filter(|info| symbol_namespace(info.symbol) == Some(namespace))
        {
            (info.install)(linker)?;
        }
        Ok(())
    }
}

impl Lib for CitizenRegistry {
    fn manifest(&self) -> LibManifest {
        LibManifest {
            id: Symbol::qualified("citizen", "explicit"),
            version: Version(env!("CARGO_PKG_VERSION").to_owned()),
            abi: AbiVersion { major: 0, minor: 1 },
            target: LibTarget::HostRegistered,
            requires: Vec::new(),
            capabilities: Vec::new(),
            exports: self
                .citizens()
                .map(|info| Export::Class {
                    symbol: parse_symbol(info.symbol),
                    class_id: None,
                })
                .collect(),
        }
    }

    fn load(&self, _cx: &mut LoadCx, linker: &mut Linker<'_>) -> Result<()> {
        self.install_all(linker)
    }
}

/// Iterates every [`CitizenInfo`] registered through `inventory`.
pub fn registered_citizens() -> impl Iterator<Item = &'static CitizenInfo> {
    inventory::iter::<CitizenInfo>.into_iter()
}

/// Iterates every [`NonCitizenInfo`] registered through `inventory`.
pub fn registered_non_citizens() -> impl Iterator<Item = &'static NonCitizenInfo> {
    inventory::iter::<NonCitizenInfo>.into_iter()
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
