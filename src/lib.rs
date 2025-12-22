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
use crate::source::DataSource;

#[pyclass]
struct FederatedStreamer {
  sources: Vec<DataSource>,
  batch_size: usize
}

#[pymethods]
impl FederatedStreamer {
  #[new]
  pub fn new(batch_size: usize, sources: Vec<DataSource>) -> PyResult<Self> {
    Ok(Self {
      sources,
      batch_size
    })
  }

}

#[pymodule]
fn python_rust_lib_gs(m: &Bound<'_, PyModule>) -> PyResult<()> {
  m.add_class::<FederatedStreamer>()?;
  Ok(())
}
