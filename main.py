#!/usr/bin/env/python3

from asyncio import run, sleep, CancelledError

import python_rust_lib_gs as rpl
from python_rust_lib_gs import FederatedStreamer, CSVSource
import pyarrow

## Create the 'mimiciv' database and the 'mimiciv_hosp' schema within it. Then:
# DROP TABLE IF EXISTS mimiciv_hosp.patients;
# CREATE TABLE mimiciv_hosp.patients
# (
#     subject_id INTEGER NOT NULL,
# gender CHAR(1) NOT NULL,
# anchor_age SMALLINT,
# anchor_year SMALLINT NOT NULL,
# anchor_year_group VARCHAR(20) NOT NULL,
# dod DATE
# );
#
# COPY mimiciv_hosp.patients
# FROM '/data/mimic-patients-B.csv'
# DELIMITER ','
# CSV HEADER;


streamer = FederatedStreamer(
    [
        CSVSource('./mimic-patients-A.csv', [('gender', 'g'), ('anchor_age', None), ('dod', 'date_of_death')], batch_size=5)
    ]
)

async def async_iter():
    try:
        async for batch in streamer:
            print(batch)
            await sleep(1.)
    except CancelledError:
        print('\nStream interrupted.')


run(async_iter())
