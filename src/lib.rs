use std::error::Error;
use futures::stream::{BoxStream, StreamExt};
use pyo3::types::PyAny;
use pyo3::{Bound, PyResult, Python, pyclass, pymethods, pymodule, types::{PyModule, PyModuleMethods}, PyErr, pyfunction, CastIntoError, Py};
use std::sync::{Arc, OnceLock};
use anyhow::__private::kind::TraitKind;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;

mod source;
use crate::source::{Source, CSVSource, DataSource};


#[pymodule]
fn python_rust_lib_gs(m: &Bound<'_, PyModule>) -> PyResult<()> {
  m.add_class::<FederatedStreamer>()?;
  m.add_class::<CSVSource>()?;

  Ok(())
}


#[pyclass]
struct FederatedStreamer {
  receiver: Arc<tokio::sync::Mutex<mpsc::Receiver<Vec<String>>>>,
  batch_size: usize,
}

#[pymethods]
impl FederatedStreamer {
  pub fn salut(&self) -> String {
    format!("salut with {} as batch size", self.batch_size)
  }
  #[new]
  #[pyo3(signature = (batch_size, sources, buffer_size=100))]
  pub fn new(batch_size: usize, sources: Vec<DataSource>, buffer_size: usize) -> PyResult<Self> {
    let (tx, rx) = mpsc::channel(buffer_size);

    // get_runtime uses get_or_init (Love it!)
    get_runtime().spawn(async move {
      // Create a stream for each source:
      let streams = sources.into_iter().map(|source| handle_batch(source, batch_size));

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

        // The try_send approach won't work since, at first, the producer will overflow the channel. Let's go for a more natural backpressure
        if tx.send(lines).await.is_err() { // Can only fail due to a send op through a closed channel
          eprintln!("Data is being sent through a full buffered channel. This is a bug!");
          std::process::exit(1);
        }
        
        // // if tx.try_send(lines).is_err() {  // That means the channel is closed because FederatedStreamer has been
        // //   // destroyed by python's garbage collector
        // //   // Alternative: cancellationtoken, but the current impl is non-blocking
        // //   break;
        // // }
        // match tx.try_send(lines) {
        //   Ok(_) => {}
        //   Err(TrySendError::Full(_)) => panic!("Data is being sent through a full buffered channel. This is a bug!"),
        //   Err(TrySendError::Closed(_)) => {
        //     /*
        //       That's ok, Python garbage-collected the FederatedStreamer instance. R.I.P.
        //     */
        //   }
        // }
      }
    });

    Ok(Self {
      receiver: Arc::new(tokio::sync::Mutex::new(rx)),
      batch_size,
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

fn handle_batch<'a>(source: DataSource, batch_size: usize) -> BoxStream<'a, anyhow::Result<Vec<String>>> {
  // https://docs.rs/futures/latest/futures/stream/fn.unfold.html
  // Unfold accepts a T and a FnMut(T) -> Future and returns a Future with an output of Option<Item, T>
  futures::stream::unfold(source, move |source| async move {
    let batch = source.fetch(Some(batch_size)).await;
    match batch {
      Ok(Some(data)) => Some((Ok(data), source)),
      Ok(None) => None, // Stream finished
      Err(e) => Some((Err(e), source)),
    }
  }).boxed()  // Not a big deal performance-wise since we're running IO-bound tasks
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
