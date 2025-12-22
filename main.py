#!/usr/bin/env/python3

import python_rust_lib_gs as rpl

print(dir(rpl))

streamer = rpl.FederatedStreamer(
    64,
    [
        ('./mimic-patients.csv', rpl.DataSource((), DataSource.MimicCSVSource))
    ]
)

print(streamer.message)
