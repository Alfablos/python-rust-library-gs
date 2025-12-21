use pyo3::{
  Bound,
  PyResult,
  pyclass,
  pymethods, // provides the `new` attribute: https://pyo3.rs/v0.27.2/class.html#constructor
  pymodule,
  types::{PyModule, PyModuleMethods},
};

#[pyclass]
struct FederatedStreamer {
  message: String,
}

#[pymethods]
impl FederatedStreamer {
  #[new]
  pub fn new() -> Self {
    Self {
      message: "Hey you!".into(),
    }
  }

  #[getter]
  pub fn message(&self) -> String {
    self.message.clone()
  }
}

/// A Python module implemented in Rust.
#[pymodule]
fn python_rust_lib_gs(m: &Bound<'_, PyModule>) -> PyResult<()> {
  m.add_class::<FederatedStreamer>()?;
  Ok(())
}
