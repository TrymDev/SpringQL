//! Objects to model pipelines.
//!
//! Created by sql-processor.
//! Executed by stream-engine.

// TODO remove this. All must reside in stream-engine. sql-processor produces stream_engine::Command.

pub(crate) mod column;
pub(crate) mod name;
pub(crate) mod option;
pub(crate) mod query_plan;
pub(crate) mod sql_type;