#[cfg(feature = "std")]
fn main() {
    // Use legacy wasm target (wasm32-unknown-unknown) instead of wasm32v1-none
    // because some dependencies don't properly support the stricter wasm32v1-none target.
    std::env::set_var("WASM_BUILD_LEGACY_TARGET", "1");
    substrate_wasm_builder::WasmBuilder::new()
        .with_current_project()
        .export_heap_base()
        .import_memory()
        .build()
}

/// The wasm builder is deactivated when compiling
/// this crate for wasm to speed up the compilation.
#[cfg(not(feature = "std"))]
fn main() {}
