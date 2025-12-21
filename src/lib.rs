use std::sync::Arc;
use pyo3::{
  Bound,
  PyResult,
  pyclass,
  pymethods, // provides the `new` attribute: https://pyo3.rs/v0.27.2/class.html#constructor
  pymodule,
  types::{PyModule, PyModuleMethods},
};

mod source;
use source::Source;


#[pyclass]
struct FederatedStreamer {
  sources: Vec<Arc<dyn Source>>,
  batch_size: usize
}

#[pymethods]
impl FederatedStreamer {
  #[new]
  pub fn new(batch_size: usize) -> Self {
    Self {
      sources: Vec::new(),
      batch_size
    }
  }

}

/// A Python module implemented in Rust.
#[pymodule]
fn python_rust_lib_gs(m: &Bound<'_, PyModule>) -> PyResult<()> {
  m.add_class::<FederatedStreamer>()?;
  Ok(())
}
