use futures::stream::StreamExt;
use pyo3::exceptions::PyOSError;
use pyo3::types::PyAny;
use pyo3::{
  Bound, PyResult, Python, pyclass, pymethods, pymodule,
  types::{PyModule, PyModuleMethods},
};
use std::sync::{Arc, OnceLock};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

mod source;
use crate::source::{CSVSource, DataSource};
use source::Source;

#[pyclass]
struct FederatedStreamer {
  receiver: Arc<tokio::sync::Mutex<mpsc::Receiver<Vec<String>>>>,
  batch_size: usize,
}

#[pymethods]
impl FederatedStreamer {
  #[new]
  #[pyo3(signature = (batch_size, sources, buffer_size=100))]
  pub fn new(batch_size: usize, sources: Vec<DataSource>, buffer_size: usize) -> PyResult<Self> {
    let (tx, rx) = mpsc::channel(buffer_size);

    // get_runtime uses get_or_init (Love it!)
    get_runtime().spawn(async move {
      // Create a stream for each source:
      let streams = sources.into_iter().map(|source| {
        // https://docs.rs/futures/latest/futures/stream/fn.unfold.html
        futures::stream::unfold(source, move |source| async move {
          let batch = source.fetch(Some(batch_size)).await;
          match batch {
            Ok(Some(data)) => {
              // Strings for now, Arrow data then
              let item = format!(
                "Fetched {batch_size} values from {}:\n{:?}",
                source.name(),
                data
              );
              Some((Ok(item), source))
            }
            Ok(None) => None, // Stream finished
            Err(e) => Some((Err(e), source)),
          }
        })
      });

      // Combine streams
      let mut united_stream = futures::stream::select_all(streams);

      // Push to channel
      while let Some(result) = united_stream.next().await {
        // We are sending Vec<String> to simulate batch
        // Convert result (PyResult<String>) to Vec<String> for now?
        // Wait, DataSource::fetch returns Result<Option<RecordBatch>>.
        // Our unfold yields PyResult<String>.
        // Usage of `sender` expects `Vec<String>`.

        match result {
          Ok(item) => {
            if tx.send(vec![item]).await.is_err() {
              break;
            }
          }
          Err(e) => {
            // How to handle error?
            // Send it? Or log it?
            // Channel expects Vec<String>.
            // We might need to change Channel type to Result<Vec<String>> later.
            // For now, let's just log print (since we can't send error easily without changing type)
            eprintln!("Error fetching data: {}", e);
          }
        }
      }
    });

    Ok(Self {
      receiver: Arc::new(tokio::sync::Mutex::new(rx)),
      batch_size,
    })
  }

  pub fn next<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
    let receiver = self.receiver.clone();
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
      let mut rx = receiver.lock().await;
      match rx.recv().await {
        Some(batch) => Ok(Some(batch)), // Returns list[str] (Vec<String> -> list)
        None => Ok(None),
      }
    })
  }
}

#[pymodule]
fn python_rust_lib_gs(m: &Bound<'_, PyModule>) -> PyResult<()> {
  m.add_class::<FederatedStreamer>()?;
  m.add_class::<CSVSource>()?;

  Ok(())
}

fn get_runtime() -> &'static Runtime {
  static RUNTIME: OnceLock<Runtime> = OnceLock::new();
  RUNTIME.get_or_init(|| {
    tokio::runtime::Builder::new_multi_thread()
      .enable_all()
      .build()
      .expect("Failed to create global Tokio runtime")
  })
}
