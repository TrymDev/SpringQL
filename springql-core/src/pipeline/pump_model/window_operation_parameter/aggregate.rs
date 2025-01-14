// This file is part of https://github.com/SpringQL/SpringQL which is licensed under MIT OR Apache-2.0. See file LICENSE-MIT or LICENSE-APACHE for full license details.

use crate::expr_resolver::expr_label::{AggrExprLabel, ValueExprLabel};

/// [GROUP BY c1, c2, c3...]
#[derive(Clone, PartialEq, Debug, Default, new)]
pub(crate) struct GroupByLabels(
    /// Empty when GROUP BY clause is not supplied.
    Vec<ValueExprLabel>,
);
impl GroupByLabels {
    pub(crate) fn as_labels(&self) -> &[ValueExprLabel] {
        &self.0
    }
}

/// TODO [support complex expression with aggregations](https://gh01.base.toyota-tokyo.tech/SpringQL-internal/SpringQL/issues/152)
///
/// ```sql
/// SELECT group_by, aggr_expr.func(aggr_expr.aggregated)
///   FROM s
///   [GROUP BY group_by]
///   SLIDING WINDOW ...;
/// ```
#[derive(Clone, PartialEq, Debug, new)]
pub(crate) struct AggregateParameter {
    // TODO multiple aggr_expr
    pub(crate) aggr_func: AggregateFunctionParameter,
    pub(crate) aggr_expr: AggrExprLabel,
    pub(crate) group_by: GroupByLabels,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum AggregateFunctionParameter {
    Avg,
}
