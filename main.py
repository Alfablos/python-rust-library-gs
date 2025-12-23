#!/usr/bin/env/python3

from asyncio import run, sleep

import python_rust_lib_gs as rpl
from python_rust_lib_gs import FederatedStreamer, CSVSource

print(dir(rpl.FederatedStreamer))



streamer = FederatedStreamer(
    64,
    [
        CSVSource('./mimic-patients.csv', ['gender', 'anchor_age', 'dod'])
    ]
)

async def async_iter():
    async for batch in streamer:
        print(batch);
        await sleep(1.)


run(async_iter())



print('Great! Now the rust library dies!')