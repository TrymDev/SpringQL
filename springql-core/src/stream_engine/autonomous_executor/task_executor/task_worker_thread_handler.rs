//! Task execution logics commonly used by GenericWorkerThread and SourceWorkerThread.

use std::{sync::Arc, thread, time::Duration};

use crate::stream_engine::autonomous_executor::{
    performance_metrics::PerformanceMetrics, pipeline_derivatives::PipelineDerivatives,
    repositories::Repositories, task::task_context::TaskContext, task_graph::task_id::TaskId,
    worker::worker_thread::WorkerThread,
};

use super::{scheduler::Scheduler, task_executor_lock::TaskExecutorLock};

// TODO config
const TASK_WAIT_MSEC: u64 = 100;

#[derive(Debug)]
pub(super) struct TaskWorkerThreadHandler;

#[derive(Debug, new)]
pub(in crate::stream_engine::autonomous_executor) struct TaskWorkerThreadArg {
    task_executor_lock: Arc<TaskExecutorLock>,
    repos: Arc<Repositories>,
}

#[derive(Debug, Default)]
pub(super) struct TaskWorkerLoopState<S: Scheduler> {
    pub(super) pipeline_derivatives: Arc<PipelineDerivatives>,
    pub(super) metrics: Arc<PerformanceMetrics>,
    pub(super) scheduler: S,
}

impl TaskWorkerThreadHandler {
    pub(super) fn main_loop_cycle<T, S>(
        current_state: TaskWorkerLoopState<S>,
        thread_arg: &TaskWorkerThreadArg,
    ) -> TaskWorkerLoopState<S>
    where
        T: WorkerThread,
        S: Scheduler,
    {
        let task_executor_lock = &thread_arg.task_executor_lock;

        if let Ok(_lock) = task_executor_lock.try_task_execution() {
            let task_series = current_state.scheduler.next_task_series(
                current_state.pipeline_derivatives.task_graph(),
                current_state.metrics.as_ref(),
            );
            if !task_series.is_empty() {
                Self::execute_task_series::<T, S>(&task_series, &current_state, thread_arg);
            } else {
                thread::sleep(Duration::from_millis(TASK_WAIT_MSEC));
            }
        }

        current_state
    }

    fn execute_task_series<T, S>(
        task_series: &[TaskId],
        current_state: &TaskWorkerLoopState<S>,
        thread_arg: &TaskWorkerThreadArg,
    ) where
        T: WorkerThread,
        S: Scheduler,
    {
        for task_id in task_series {
            let context = TaskContext::new(
                task_id.clone(),
                current_state.pipeline_derivatives.clone(),
                thread_arg.repos.clone(),
            );

            let task = current_state
                .pipeline_derivatives
                .get_task(task_id)
                .expect("task id got from scheduler");

            task.run(&context).unwrap_or_else(T::handle_error);
        }
    }
}
