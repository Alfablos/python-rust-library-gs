use anyhow::Result;
use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
use std::path::Path;

use polars::prelude::{Expr, LazyCsvReader, LazyFileListReader, LazyFrame, col};
use pyo3::exceptions::{PyFileNotFoundError, PyOSError, PyTypeError};
use pyo3::{Borrowed, FromPyObject, PyAny, PyErr, PyResult, pyclass, pymethods};

pub struct RecordBatch;

#[async_trait]
#[enum_dispatch(DataSource)]
pub trait Source: Send + Sync {
  fn name(&self) -> &'static str;
  async fn fetch(&self, batch_size: Option<usize>) -> Result<Option<Vec<String>>>;
}

#[enum_dispatch]
// Clone must be implemented for FederatedStreamer::new to accept a Vec<DataSource>: https://docs.rs/pyo3/latest/pyo3/conversion/trait.FromPyObject.html#implementors
// impl<'a, 'py, T> FromPyObject<'a, 'py> for T where T: PyClass + Clone + ExtractPyClassWithClone
#[derive(Clone, FromPyObject)]
pub enum DataSource {
  CSV(CSVSource),
}

#[pyclass]
#[derive(Clone)]
pub struct CSVSource {
  data: LazyFrame,
}

#[pymethods]
impl CSVSource {
  #[new]
  pub fn new(path: &str, fields: Vec<String>) -> PyResult<Self> {
    let data = LazyCsvReader::new(path)
      .with_has_header(true)
      .finish()
      .map_err(|e| PyOSError::new_err(format!("Unable to read from path {path}: {}", e)))?;
    let data = data.select(fields.into_iter().map(|f| col(f)).collect::<Vec<Expr>>());
    Ok(Self { data })
  }
}

#[async_trait]
impl Source for CSVSource {
  fn name(&self) -> &'static str {
    "MIMIC IV - CSV"
  }

  async fn fetch(&self, batch_size: Option<usize>) -> Result<Option<Vec<String>>> {
    // Match trait signature
    // Placeholder implementation returning String
    Ok(Some(vec!["Simulated Batch Data".to_string()]))
  }
}
