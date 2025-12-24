use anyhow::Result;
use async_trait::async_trait;

use polars::prelude::{Expr, LazyCsvReader, LazyFileListReader, LazyFrame, col, SchemaRef, Schema};
use pyo3::exceptions::{PyFileNotFoundError, PyOSError, PyTypeError};
use pyo3::{Borrowed, FromPyObject, PyAny, PyErr, PyResult, pyclass, pymethods};

use arrow::array::RecordBatch as ArrowRecordBatch;

pub mod csv;

use csv::CSVSource;

#[async_trait]
pub trait Source: Send + Sync {
  fn name(&self) -> &'static str;
  async fn fetch(&self, batch_size: Option<usize>) -> Result<Option<ArrowRecordBatch>>;
}

// Clone must be implemented for FederatedStreamer::new to accept a Vec<DataSource>: https://docs.rs/pyo3/latest/pyo3/conversion/trait.FromPyObject.html#implementors
// impl<'a, 'py, T> FromPyObject<'a, 'py> for T where T: PyClass + Clone + ExtractPyClassWithClone
#[derive(Clone, FromPyObject)]
pub enum DataSource {
  CSV(CSVSource),
}

#[async_trait]
impl Source for DataSource {
  fn name(&self) -> &'static str {
    match &self {
      DataSource::CSV(_) => "CSV"
    }
  }

  async fn fetch(&self, batch_size: Option<usize>) -> Result<Option<ArrowRecordBatch>> {
    match &self {
      DataSource::CSV(c) => {
        todo!()
      }
      _ => Err(anyhow::Error::msg("Unsupported data source"))
    }
  }
}



