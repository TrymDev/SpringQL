// This file is part of https://github.com/SpringQL/SpringQL which is licensed under MIT OR Apache-2.0. See file LICENSE-MIT or LICENSE-APACHE for full license details.

use self::alter_pipeline_command::AlterPipelineCommand;

pub(crate) mod alter_pipeline_command;
pub(crate) mod insert_plan;
pub(crate) mod query_plan;

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum Command {
    AlterPipeline(AlterPipelineCommand),
}
