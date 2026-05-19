//! Quicko2 FFI — PyO3 (Python) and UniFFI (Kotlin/Swift) bindings.

// UniFFI proc-macro scaffolding (replaces UDL-based include_scaffolding!)
uniffi::setup_scaffolding!();

pub mod uniffi_bridge;
pub use uniffi_bridge::*;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[cfg(feature = "pyo3")]
#[pymodule]
fn quicko2_core(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    // TODO: expose Python bindings here
    Ok(())
}
