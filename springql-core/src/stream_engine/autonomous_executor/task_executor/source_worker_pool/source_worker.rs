// This file is part of https://github.com/SpringQL/SpringQL which is licensed under MIT OR Apache-2.0. See file LICENSE-MIT or LICENSE-APACHE for full license details.

pub(in crate::stream_engine::autonomous_executor) mod source_worker_thread;

use std::sync::Arc;

use crate::stream_engine::autonomous_executor::{
    args::{Coordinators, EventQueues},
    main_job_lock::MainJobLock,
    task_executor::task_worker_thread_handler::TaskWorkerThreadArg,
    worker::worker_handle::WorkerHandle,
};

use self::source_worker_thread::SourceWorkerThread;

/// Worker to execute pump and sink tasks.
#[derive(Debug)]
pub(super) struct SourceWorker {
    _handle: WorkerHandle,
}

impl SourceWorker {
    pub(super) fn new(
        main_job_lock: Arc<MainJobLock>,
        event_queues: EventQueues,
        coordinators: Coordinators,
        thread_arg: TaskWorkerThreadArg,
    ) -> Self {
        let handle = WorkerHandle::new::<SourceWorkerThread>(
            main_job_lock,
            event_queues,
            coordinators,
            thread_arg,
        );
        Self { _handle: handle }
    }
}
