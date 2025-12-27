use anyhow::__private::kind::TraitKind;
use futures::stream::{BoxStream, StreamExt};
use pyo3::types::PyAny;
use pyo3::{
  Bound, Py, PyResult, Python, pyclass, pymethods, pymodule,
  types::{PyModule, PyModuleMethods},
};
use std::error::Error;
use std::sync::{Arc, OnceLock};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
// use tokio::sync::mpsc::error::TrySendError;

mod source;
use crate::source::{DataSource, Source, csv::CSVSource};

#[pymodule]
fn python_rust_lib_gs(m: &Bound<'_, PyModule>) -> PyResult<()> {
  m.add_class::<FederatedStreamer>()?;
  m.add_class::<CSVSource>()?;

  Ok(())
}

#[pyclass]
struct FederatedStreamer {
  receiver: Arc<tokio::sync::Mutex<mpsc::Receiver<Py<PyAny>>>>,
}

#[pymethods]
impl FederatedStreamer {
  #[new]
  pub fn new(sources: Vec<DataSource>) -> PyResult<Self> {
    let (tx, rx) = mpsc::channel(2 * sources.len());

    // get_runtime uses get_or_init (Love it!)
    get_runtime().spawn(async move {
      // Create a stream for each source:
      let streams = sources
        .into_iter()
        .map(|source| handle_batch(source));

      // Combine streams
      let mut united_stream = futures::stream::select_all(streams);

      // Push to channel
      while let Some(result) = united_stream.next().await {
        let lines = match result {
          Ok(lines) => lines,
          Err(e) => {
            eprintln!("Unable to fetch more data: {e}");
            std::process::exit(1);
          }
        };

        // If sending through a closed channel it means the work is done: python runtime has been destroyed
        if tx.send(lines).await.is_err() { // Can only fail due to a send op through a closed channel, which means the job is done
          break;
        }
      }
    });

    Ok(Self {
      receiver: Arc::new(tokio::sync::Mutex::new(rx)),
    })
  }

  pub fn __aiter__<'py>(slf: Py<Self>) -> Py<Self> {
    slf
  }

  pub fn __anext__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
    let receiver = self.receiver.clone();
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
      let mut rx = receiver.lock().await;
      match rx.recv().await {
        Some(batch) => Ok(Some(batch)),
        None => Ok(None),
      }
    })
  }
}

fn handle_batch<'a>(
  source: DataSource
) -> BoxStream<'a, PyResult<Py<PyAny>>> {
  // https://docs.rs/futures/latest/futures/stream/fn.unfold.html
  // Unfold accepts a T and a FnMut(T) -> Future and returns a Future with an output of Option<Item, T>
  futures::stream::unfold(source, move |source| async move {
    let batch = source.fetch(source.batch_size()).await;
    match batch {
      Ok(Some(data)) => Some((Ok(data), source)),
      Ok(None) => None, // Stream finished
      Err(e) => Some((Err(e), source)),
    }
  })
  .boxed() // Not a big deal performance-wise since we're running IO-bound tasks
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
