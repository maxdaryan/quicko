fn main() {
    #[cfg(feature = "pyo3")]
    pyo3_build_config::add_extension_module_link_args();

    // UniFFI scaffolding is now handled via proc-macros (uniffi::setup_scaffolding!),
    // so no UDL-based generate_scaffolding() call is needed.
}
