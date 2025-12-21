use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use enum_dispatch::enum_dispatch;

use polars::prelude::LazyFrame;

pub struct RecordBatch;

#[async_trait]
#[enum_dispatch(DataSource)]
pub (crate) trait Source: Send + Sync {
    // fn name() -> &'static str;
    async fn fetch(&self, batch_size: Option<usize>) -> Result<Option<RecordBatch>>;
}

#[enum_dispatch]
pub (crate) enum DataSource {
    MimicCSV(MimicCSVSource)
}


pub (crate) struct MimicCSVSource {}

impl MimicCSVSource {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let x = LazyFrame::lazy()
    }
}

#[async_trait]
impl Source for MimicCSVSource {
    async fn fetch(&self, batch_size: Option<usize>) -> Result<Option<RecordBatch>> {
        todo!()
    }
}

