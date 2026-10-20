use std::sync::OnceLock;

use jni::JNIEnv;
use thiserror::Error;

use crate::mapping::client::minecraft::Minecraft;
use crate::mapping::Mapping;
use crate::module::registry::ModuleRegistry;

static MAPPING: OnceLock<Mapping> = OnceLock::new();

static CLIENT: OnceLock<Client> = OnceLock::new();

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("the client is already initialized")]
    AlreadyInitialized,

    #[error(transparent)]
    Init(#[from] anyhow::Error),
}

pub fn init() -> Result<(), ClientError> {
    MAPPING
        .set(Mapping::new()?)
        .map_err(|_| ClientError::AlreadyInitialized)?;
    CLIENT
        .set(Client::new()?)
        .map_err(|_| ClientError::AlreadyInitialized)?;
    Ok(())
}

#[inline]
pub fn mapping() -> &'static Mapping {
    MAPPING.get().expect("mapping() used before client init")
}

/// The global client. Valid once [`init`] has succeeded.
#[inline]
pub fn client() -> &'static Client {
    CLIENT.get().expect("client() used before client init")
}

#[inline]
pub fn try_client() -> Option<&'static Client> {
    CLIENT.get()
}

/// The running Minecraft game. Valid once [`init`] has succeeded.
#[inline]
pub fn minecraft() -> &'static Minecraft {
    &client().minecraft
}

pub fn env() -> anyhow::Result<JNIEnv<'static>> {
    mapping().get_env()
}

/// Releases the JVM resources the global state holds — called from
/// `cleanup_client` before the library is unloaded.
///
/// The `MAPPING` / `CLIENT` statics are `OnceLock`s and are never dropped, so
/// the handful of global references kept in plain fields (`Minecraft` and
/// `Window`) outlive a hot-reload. Those point at session-lifetime singletons,
/// so only a couple of global-ref table slots leak per reload; everything
/// sizable — the class-handle cache, the game class loader, the cached player
/// — is released here.
pub fn teardown() {
    // Remove the Netty pipeline handler first — leaving it in place would
    // crash the JVM once this library is unloaded.
    crate::net::teardown();
    if let Some(client) = CLIENT.get() {
        client.minecraft.teardown();
    }
    if let Some(mapping) = MAPPING.get() {
        mapping.teardown();
    }
}

/// The running game and module state.
pub struct Client {
    minecraft: Minecraft,
    /// Every registered module.
    pub modules: ModuleRegistry,
}

impl Client {
    /// Builds the client. The [`MAPPING`] global must already be set.
    fn new() -> Result<Client, ClientError> {
        Ok(Client {
            minecraft: Minecraft::new()?,
            modules: ModuleRegistry::new(),
        })
    }
}
