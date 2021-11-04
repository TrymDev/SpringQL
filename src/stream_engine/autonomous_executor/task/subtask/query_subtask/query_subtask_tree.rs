mod query_subtask_node;

use std::rc::Rc;
use std::sync::Arc;

use self::query_subtask_node::collect_subtask::CollectSubtask;
use self::query_subtask_node::QuerySubtaskNode;

use super::final_row::FinalRow;
use super::interm_row::NewRow;
use crate::error::Result;
use crate::stream_engine::autonomous_executor::row::Row;
use crate::stream_engine::autonomous_executor::task::task_context::TaskContext;
use crate::stream_engine::command::query_plan::query_plan_node::operation::LeafOperation;
use crate::stream_engine::command::query_plan::query_plan_node::QueryPlanNode;
use crate::stream_engine::command::query_plan::QueryPlan;
use crate::stream_engine::dependency_injection::DependencyInjection;

#[derive(Debug)]
pub(super) struct QuerySubtaskTree {
    root: QuerySubtaskNode,

    /// Some(_) means: Output of the query plan is this NewRow.
    /// None means: Output of the query plan is the input of it.
    latest_new_row: Option<NewRow>,
}

impl QuerySubtaskTree {
    pub(super) fn compile(query_plan: QueryPlan) -> Self {
        let plan_root = query_plan.root();
        let root = Self::compile_node(plan_root);

        Self {
            root,
            latest_new_row: None,
        }
    }

    fn compile_node(plan_node: Rc<QueryPlanNode>) -> QuerySubtaskNode {
        match plan_node.as_ref() {
            QueryPlanNode::Leaf(leaf_node) => match &leaf_node.op {
                LeafOperation::Collect => QuerySubtaskNode::Collect(CollectSubtask::new()),
            },
        }
    }

    /// # Failure
    ///
    /// - [SpringError::InputTimeout](crate::error::SpringError::InputTimeout) when:
    ///   - Input from a source stream is not available within timeout period.
    pub(super) fn run<DI: DependencyInjection>(
        &mut self,
        context: &TaskContext<DI>,
    ) -> Result<FinalRow> {
        let row = Self::run_dfs_post_order::<DI>(&self.root, &mut self.latest_new_row, context)?;

        if let Some(new_row) = self.latest_new_row.take() {
            Ok(FinalRow::NewlyCreated(new_row.into()))
        } else {
            Ok(FinalRow::Preserved(row))
        }
    }

    fn run_dfs_post_order<DI: DependencyInjection>(
        executor: &QuerySubtaskNode,
        latest_new_row: &mut Option<NewRow>,
        context: &TaskContext<DI>,
    ) -> Result<Arc<Row>> {
        match executor {
            QuerySubtaskNode::Collect(collect_subtask) => collect_subtask.run::<DI>(context),
            QuerySubtaskNode::Stream(_) => todo!(),
            QuerySubtaskNode::Window(_) => todo!(),
        }
    }
}