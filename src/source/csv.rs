use arrow::array::{ArrayData, ArrayRef, make_array};
use arrow::datatypes::{Field, Schema};
use arrow::ffi::{FFI_ArrowArray, FFI_ArrowSchema, from_ffi};
use arrow::pyarrow::ToPyArrow;
use arrow::record_batch::RecordBatch as ArrowBatch;
use polars::prelude::{
  CompatLevel, Expr, LazyCsvReader, LazyFileListReader, LazyFrame, PlPath, SchemaExt, col,
};
use polars_arrow::ffi::{ArrowArray, ArrowSchema, export_array_to_c, export_field_to_c};
use pyo3::exceptions::PyOSError;
use pyo3::{IntoPyObject, Py, PyAny, PyResult, Python, pyclass, pymethods};
use std::sync::Arc;
use tokio::sync::Mutex;

#[pyclass]
#[derive(Clone)]
pub struct CSVSource {
  path: String,
  data: LazyFrame,
  offset: Arc<Mutex<usize>>,
  batch_size: Option<usize>,
}

#[pymethods]
impl CSVSource {
  #[new]
  #[pyo3(signature = (path, fields, batch_size=None))]
  pub fn new(path: &str, fields: Vec<String>, batch_size: Option<usize>) -> PyResult<Self> {
    let data = LazyCsvReader::new(PlPath::new(path))
      .with_has_header(true)
      .finish()
      .map_err(|e| PyOSError::new_err(format!("Unable to read from path {path}: {}", e)))?;
    let data = data.select(fields.into_iter().map(|f| col(f)).collect::<Vec<Expr>>());
    Ok(Self {
      data,
      path: String::from(path),
      offset: Arc::new(Mutex::new(0)),
      batch_size,
    })
  }

  pub async fn override_offset(&self, new_offset: usize) {
    let mut offset = self.offset.lock().await;
    *offset = new_offset;
  }

  pub async fn read_next(&self) -> PyResult<Option<Py<PyAny>>> {
    let data = self.data.clone(); // just cloning the reader

    if let Some(batch_size) = self.batch_size {
      let current_offset = *self.offset.lock().await;
      
      // It seems Polars doesn't support "collect"-ing asynchronously.
      // Since we cannot block we need spawn_blocking :/
      let data = data.clone();
      let df = tokio::task::spawn_blocking(move || {
          data
            .slice(current_offset as i64, batch_size as u32)
            .collect()
            .map_err(|e| PyOSError::new_err(e.to_string()))
      })
      .await
      .map_err(|e| PyOSError::new_err(format!("Error reading from {}: {e}", &self.path)))??;


     self.override_offset(current_offset + batch_size).await;

      if df.height() == 0 {
        return Ok(None);
      }

      // In order to maintin a zero-copy approach we'll use FFI to reference
      // the Polars df that's already in memory from pyarrow
      // https://docs.rs/polars-arrow/latest/polars_arrow/ffi/fn.export_array_to_c.html

      // let df_schema = df.schema().to_arrow(CompatLevel::newest());

      let mut arrow_payload: Vec<ArrayRef> = Vec::with_capacity(df.get_columns().len());
      let mut arrow_fields: Vec<Field> = Vec::with_capacity(df.get_columns().len());

      for series in df.get_columns().iter() {
        /*
          Polars uses the same memory representation as Arrow for arrays and fields as it's backed by it.
          We then convert polars -> ffi -> arrow (TODO: check again if there's anything more straightforward)
        */

        // data
        let p_series_as_arr_array = series
          .clone() // cheap clone increments an Arc reference count
          .rechunk_to_arrow(CompatLevel::newest());
        // schema
        let p_series_schema = series.field().to_arrow(CompatLevel::newest());

        // ffi data
        let polars_ffi_array = export_array_to_c(p_series_as_arr_array);
        // ffi schema
        let polars_ffi_schema = export_field_to_c(&p_series_schema);

        let arrow_ffi_array: FFI_ArrowArray = unsafe { std::mem::transmute(polars_ffi_array) };
        let arrow_ffi_schema: FFI_ArrowSchema = unsafe { std::mem::transmute(polars_ffi_schema) };

        let arrow_data = unsafe { from_ffi(arrow_ffi_array, &arrow_ffi_schema) }.map_err(|e| {
          PyOSError::new_err(format!(
            "FFI Import Error while converting to arrow data: {}",
            e
          ))
        })?;
        let arrow_field = Field::try_from(&arrow_ffi_schema).map_err(|e| {
          PyOSError::new_err(format!(
            "FFI Import Error while converting to arrow field for file {}: {}",
            self.path,
            e
          ))
        })?;

        arrow_payload.push(make_array(arrow_data));
        arrow_fields.push(arrow_field);
      }

      let schema_ref = Arc::new(arrow::datatypes::Schema::new(arrow_fields));
      let record_batch = ArrowBatch::try_new(schema_ref, arrow_payload).map_err(|e| {
        PyOSError::new_err(format!(
          "Error creating record batch; cannot convert dataframe to an Arrow record batch for file {}: {}",
          self.path,
          e
        ))
      })?;

      Python::attach(|py| {
        let py_batch = record_batch
          .to_pyarrow(py)
          .map_err(|e| PyOSError::new_err(format!("Error converting to pyarrow: {}", e)))?;
        Ok(Some(py_batch.unbind()))
      })
    } else {
      unimplemented!("Not so fast!")
    }
  }
}
