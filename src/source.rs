use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use enum_dispatch::enum_dispatch;

use polars::prelude::{col, Expr, LazyCsvReader, LazyFileListReader, LazyFrame};
use pyo3::{pyclass, pymethods, FromPyObject, PyResult};
use pyo3::exceptions::PyFileNotFoundError;

pub struct RecordBatch;

#[async_trait]
#[enum_dispatch(DataSource)]
pub trait Source: Send + Sync {
    fn name(&self) -> &'static str;
    async fn fetch(&self, batch_size: Option<usize>) -> Result<Option<RecordBatch>>;
}


#[pyclass]
#[enum_dispatch]
// Clone must be implemented for FederatedStreamer::new to accept a Vec<DataSource>: https://docs.rs/pyo3/latest/pyo3/conversion/trait.FromPyObject.html#implementors
// impl<'a, 'py, T> FromPyObject<'a, 'py> for T where T: PyClass + Clone + ExtractPyClassWithClone
#[derive(Clone)]
pub enum DataSource {
    MimicCSV(MimicCSVSource)
}


#[pyclass]
#[derive(Clone)]
pub struct MimicCSVSource {
  data: LazyFrame
}

#[pymethods]
impl MimicCSVSource {
    #[new]
    pub fn new(path: &str, fields: Vec<String>) -> PyResult<Self> {
        let data = LazyCsvReader::new(path)
          .with_has_header(true)
          .finish()
          .map_err(|e| PyFileNotFoundError::new_err(e.to_string()))?;
        let data = data.select(fields.into_iter().map(|f| col(f)).collect::<Vec<Expr>>());
        Ok(Self { data })
    }
}



#[async_trait]
impl Source for MimicCSVSource {
  fn name(&self) -> &'static str {
    "MIMIC IV - CSV"
  }

  async fn fetch(&self, batch_size: Option<usize>) -> Result<Option<RecordBatch>> {
        todo!()
    }
}

