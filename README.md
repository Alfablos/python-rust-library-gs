# Rust library for Python example

This *minimal* project illustrates how Rust can be used to write libraries that are then exposed to a Python runtime and imported.

**Note**: the `mimic-patients.csv` file used in this example is taken from the demo MIMIC-IV dataset freely available online.


## Build & Usage

See the [**master**](https://github.com/Alfablos/python-rust-library-gs/tree/master) branch.

## Stream From File
The goal here is to read a CSV file (`./mimic-patients.csv`) using `polars` and serve data to the Python runtime in Arrow format.

The FederatedStreamer is used in Python as an async generator:
```python
async def async_iter():
    try:
        async for batch in streamer:
            print(batch.columns)
            await sleep(1.)
    except CancelledError:
        print('\nStream interrupted.')

```

A batch source is defined depending on its type, CSV files sources can be initializzed like this:
```python
streamer = FederatedStreamer(
    [
        CSVSource('./mimic-patients.csv', ['gender', 'anchor_age', 'dod'], batch_size=5)
    ]
)
```

Data, in this case, is read using polars specifying which columns should be *lazily* read. It is then turned into an Arrow RecordBatch and returned to the python runtime with a zero-copy mechanism.

Assuming more than one CSV source is defined, the rust library will poll all the configured reader (via `futures::stream::select_all(streams)`), returning the first available batch. This should **prevent slow
sources from slowing down faster ones**.