use polars::prelude::{col, Expr, LazyCsvReader, LazyFileListReader, LazyFrame};
use pyo3::{pyclass, pymethods, PyResult};
use pyo3::exceptions::PyOSError;

#[pyclass]
#[derive(Clone)]
pub struct CSVSource {
  path: String,
  data: LazyFrame,
  offset: usize,
  batch_size: Option<usize>
}

#[pymethods]
impl CSVSource {
  #[new]
  #[pyo3(signature = (path, fields, batch_size=None))]
  pub fn new(path: &str, fields: Vec<String>, batch_size: Option<usize>) -> PyResult<Self> {
    let data = LazyCsvReader::new(path)
      .with_has_header(true)
      .finish()
      .map_err(|e| PyOSError::new_err(format!("Unable to read from path {path}: {}", e)))?;
    let data = data.select(fields.into_iter().map(|f| col(f)).collect::<Vec<Expr>>());
    Ok(Self { data, path: String::from(path), offset: 0, batch_size })
  }

  pub fn read_next(&mut self) -> Result<Option<String>> {
    let data = self.data
      .clone();
      if let Some(batch_size) = self.batch_size {
        let df = self.data.slice(self.offset as i64, batch_size as u32)
          .collect()
          .map_err(|e| PyOSError::new_err(format!("Error reading from file {}: {}", &self.path, e)))?;

        if df.height() == 0 {
          return Ok(None);
        }

        self.offset += batch_size;

        return Ok(Some(df.to_ar))
      } else {

      }
  }
}
