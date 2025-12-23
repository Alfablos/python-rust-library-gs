use std::sync::Arc;
use pyo3::{Bound, PyResult, pyclass, pymethods, pymodule, types::{PyModule, PyModuleMethods}, Python};
use pyo3::exceptions::PyOSError;

mod source;
use source::Source;
use crate::source::{DataSource, CSVSource};

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
  pub async fn next<'py>(&mut self, py: Python<'py>) -> PyResult<Option<String>> {
    let batch_size = self.batch_size;

      // we need a mechanism to concurrently fetch records from all sources
      // like a some_async_magic::select_all and keep returning what fetch() returns (strings for now)

    pyo3_async_runtimes::tokio::future_into_py(py,async move {
      self.sources.get(0).unwrap().fetch(Some(self.batch_size))
      .await
      .map_err(|e| PyOSError::new_err(format!("Unable to fetch data from {}: {}", self.sources.get(0).name(), e)))
    })
  }
}

#[pymodule]
fn python_rust_lib_gs(m: &Bound<'_, PyModule>) -> PyResult<()> {
  m.add_class::<FederatedStreamer>()?;
  m.add_class::<CSVSource>()?;

  Ok(())
}
