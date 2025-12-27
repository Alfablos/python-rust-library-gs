#!/usr/bin/env/python3

from asyncio import run, sleep, CancelledError

import python_rust_lib_gs as rpl
from python_rust_lib_gs import FederatedStreamer, CSVSource
import pyarrow


streamer = FederatedStreamer(
    [
        CSVSource('./mimic-patients.csv', ['gender', 'anchor_age', 'dod'], batch_size=5)
    ]
)

async def async_iter():
    try:
        async for batch in streamer:
            print(batch.columns)
            await sleep(1.)
    except CancelledError:
        print('\nStream interrupted.')


run(async_iter())
