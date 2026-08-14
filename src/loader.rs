use libloading::{Library, Symbol};
use opsis_extension_api::{
    ExtensionCreateFn, ExtensionManifest, ExtensionRegistry, OpsisExtension,
    EXTENSION_ENTRYPOINT_SYMBOL,
};
use std::path::Path;

/// Owns a loaded native extension instance and keeps its dynamic library handle alive.
pub struct LoadedExtension {
    pub manifest: ExtensionManifest,
    pub instance: Box<dyn OpsisExtension>,
    #[allow(dead_code)]
    pub library: Library,
}

/// Dynamically load an extension from a native dynamic library path (.dll / .so / .dylib).
pub fn load_native_extension(
    binary_path: &Path,
    registry: &mut ExtensionRegistry,
) -> Result<LoadedExtension, String> {
    unsafe {
        let lib = Library::new(binary_path)
            .map_err(|e| format!("Failed to load dynamic library at {:?}: {e}", binary_path))?;

        let constructor: Symbol<ExtensionCreateFn> = lib
            .get(EXTENSION_ENTRYPOINT_SYMBOL)
            .map_err(|e| format!("Missing extension entry point symbol: {e}"))?;

        let mut instance = constructor();
        let manifest = instance.manifest();

        instance
            .on_init(registry)
            .map_err(|e| format!("Extension '{}:{}' on_init failed: {e}", manifest.id, manifest.version))?;

        Ok(LoadedExtension {
            manifest,
            instance,
            library: lib,
        })
    }
}
