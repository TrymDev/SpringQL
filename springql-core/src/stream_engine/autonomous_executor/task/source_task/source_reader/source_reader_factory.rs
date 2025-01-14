// This file is part of https://github.com/SpringQL/SpringQL which is licensed under MIT OR Apache-2.0. See file LICENSE-MIT or LICENSE-APACHE for full license details.

use crate::error::Result;
use crate::low_level_rs::SpringSourceReaderConfig;
use crate::pipeline::option::Options;
use crate::pipeline::source_reader_model::source_reader_type::SourceReaderType;
use crate::stream_engine::autonomous_executor::task::source_task::source_reader::SourceReader;

use super::net_client::NetClientSourceReader;
use super::net_server::NetServerSourceReader;

pub(in crate::stream_engine::autonomous_executor) struct SourceReaderFactory;

impl SourceReaderFactory {
    pub(in crate::stream_engine::autonomous_executor) fn source(
        source_reader_type: &SourceReaderType,
        options: &Options,
        config: &SpringSourceReaderConfig,
    ) -> Result<Box<dyn SourceReader>> {
        match source_reader_type {
            SourceReaderType::NetClient => {
                Ok(Box::new(NetClientSourceReader::start(options, config)?))
            }
            SourceReaderType::NetServer => {
                Ok(Box::new(NetServerSourceReader::start(options, config)?))
            }
        }
    }
}
