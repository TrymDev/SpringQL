// This file is part of https://github.com/SpringQL/SpringQL which is licensed under MIT OR Apache-2.0. See file LICENSE-MIT or LICENSE-APACHE for full license details.

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use parking_lot::RwLock;

use crate::stream_engine::autonomous_executor::task_graph::queue_id::row_queue_id::RowQueueId;

use super::row_queue::RowQueue;

#[derive(Debug, Default)]
pub(in crate::stream_engine::autonomous_executor) struct RowQueueRepository {
    repo: RwLock<HashMap<RowQueueId, Arc<RowQueue>>>,
}

impl RowQueueRepository {
    pub(in crate::stream_engine::autonomous_executor) fn get(
        &self,
        row_queue_id: &RowQueueId,
    ) -> Arc<RowQueue> {
        let repo = self.repo.read();
        repo.get(row_queue_id)
            .unwrap_or_else(|| panic!("row queue id {} is not in RowQueueRepository", row_queue_id))
            .clone()
    }

    /// Removes all currently existing queues and creates new empty ones.
    pub(in crate::stream_engine::autonomous_executor) fn reset(
        &self,
        queue_ids: HashSet<RowQueueId>,
    ) {
        let mut repo = self.repo.write();
        repo.clear();

        queue_ids.into_iter().for_each(|queue_id| {
            repo.insert(queue_id, Arc::new(RowQueue::default()));
        });
    }

    pub(in crate::stream_engine::autonomous_executor) fn purge(&self) {
        let mut repo = self.repo.write();
        repo.iter_mut().for_each(|(_, queue)| {
            queue.purge();
        });
    }
}
