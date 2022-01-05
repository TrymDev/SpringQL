// Copyright (c) 2021 TOYOTA MOTOR CORPORATION. Licensed under MIT OR Apache-2.0.

mod scheduler;
mod task_executor_lock;
mod worker_pool;

use crate::error::Result;
use std::sync::Arc;

use self::{
    task_executor_lock::{PipelineUpdateLockGuard, TaskExecutorLock},
    worker_pool::WorkerPool,
};
use super::{
    current_pipeline::CurrentPipeline,
    row::row_repository::RowRepository,
    task::{
        sink_task::sink_writer::sink_writer_repository::SinkWriterRepository,
        source_task::source_reader::source_reader_repository::SourceReaderRepository,
        task_graph::TaskGraph,
    },
};

/// Task executor executes task graph's dataflow by internal worker threads.
/// Source tasks are scheduled by SourceScheduler and other tasks are scheduled by FlowEfficientScheduler (in Moderate state) or MemoryReducingScheduler (in Severe state).
///
/// All interface methods are called from main thread, while `new()` spawns worker threads.
#[derive(Debug)]
pub(in crate::stream_engine) struct TaskExecutor {
    task_executor_lock: Arc<TaskExecutorLock>,

    row_repo: Arc<RowRepository>,
    source_reader_repo: Arc<SourceReaderRepository>,
    sink_writer_repo: Arc<SinkWriterRepository>,

    worker_pool: WorkerPool,
}

impl TaskExecutor {
    pub(in crate::stream_engine::autonomous_executor) fn new(
        n_worker_threads: usize,
        current_pipeline: Arc<CurrentPipeline>,
    ) -> Self {
        let task_executor_lock = Arc::new(TaskExecutorLock::default());

        let row_repo = Arc::new(RowRepository::default());
        let source_reader_repo = Arc::new(SourceReaderRepository::default());
        let sink_writer_repo = Arc::new(SinkWriterRepository::default());

        Self {
            task_executor_lock: task_executor_lock.clone(),

            worker_pool: WorkerPool::new(
                n_worker_threads,
                task_executor_lock,
                current_pipeline,
                row_repo.clone(),
                source_reader_repo.clone(),
                sink_writer_repo.clone(),
            ),
            row_repo,
            source_reader_repo,
            sink_writer_repo,
        }
    }

    /// AutonomousExecutor acquires lock when pipeline is updated.
    pub(in crate::stream_engine::autonomous_executor) fn pipeline_update_lock(
        &self,
    ) -> PipelineUpdateLockGuard {
        self.task_executor_lock.pipeline_update()
    }

    /// Update workers' internal current pipeline.
    pub(in crate::stream_engine::autonomous_executor) fn update_pipeline(
        &self,
        _lock_guard: &PipelineUpdateLockGuard,
        current_pipeline: Arc<CurrentPipeline>,
    ) -> Result<()> {
        let pipeline = current_pipeline.pipeline();
        pipeline
            .all_sources()
            .into_iter()
            .try_for_each(|source_reader| self.source_reader_repo.register(source_reader))?;
        pipeline
            .all_sinks()
            .into_iter()
            .try_for_each(|sink_writer| self.sink_writer_repo.register(sink_writer))?;

        self.worker_pool.interrupt_pipeline_update(current_pipeline);

        Ok(())
    }

    /// Stop all source tasks and executes pump tasks and sink tasks to finish all rows remaining in queues.
    pub(in crate::stream_engine::autonomous_executor) fn cleanup(
        &self,
        _lock_guard: &PipelineUpdateLockGuard,
        task_graph: &TaskGraph,
    ) {
        // TODO do not just remove rows in queues. Do the things in doc comment.

        self.row_repo.reset(task_graph.all_tasks());
    }
}