#!/usr/bin/env/python3

import python_rust_lib_gs as rpl
from python_rust_lib_gs import FederatedStreamer, CSVSource

print(dir(rpl))



streamer = FederatedStreamer(
    64,
    [
        CSVSource('./mimic-patients.csv', ['gender', 'anchor_age', 'dod'])
    ]
)

print(streamer)
