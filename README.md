# Rust library for Python example

This *minimal* project illustrates how Rust can be used to write libraries that are then exposed to a Python runtime and imported.


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