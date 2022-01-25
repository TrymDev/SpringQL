use super::SelectSyntaxAnalyzer;
use crate::{
    error::{Result, SpringError},
    expression::Expression,
    pipeline::{
        correlation::aliased_correlation_name::AliasedCorrelationName,
        field::{
            aliased_field_name::AliasedFieldName, field_name::FieldName,
            field_pointer::FieldPointer,
        },
        name::AttributeName,
    },
    sql_processor::sql_parser::syntax::SelectFieldSyntax,
};
use anyhow::anyhow;

impl SelectSyntaxAnalyzer {
    pub(in super::super) fn aliased_field_names_in_projection(
        &self,
    ) -> Result<Vec<AliasedFieldName>> {
        let from_item_correlations = self.from_item_correlations()?;
        let select_fields = &self.select_syntax.fields;

        select_fields
            .iter()
            .map(|select_field| {
                Self::select_field_into_aliased_field_name(select_field, &from_item_correlations)
            })
            .collect::<Result<_>>()
    }

    fn select_field_into_aliased_field_name(
        select_field: &SelectFieldSyntax,
        from_item_correlations: &[AliasedCorrelationName],
    ) -> Result<AliasedFieldName> {
        match &select_field.expression {
            Expression::Constant(_) => {
                unimplemented!("constant in select field is not supported currently",)
            }
            Expression::UnaryOperator(_, _) | Expression::BooleanExpr(_) => {
                // TODO Better to shrink expression in this layer.
                unimplemented!("unary/binary operation in select field is not supported currently",)
            }
            Expression::FieldPointer(ptr) => {
                let field_name = Self::field_name(ptr, from_item_correlations)?;
                let afn = AliasedFieldName::new(field_name, select_field.alias.clone());
                Ok(afn)
            }
        }
    }

    /// TODO may need Pipeline when:
    /// - pointer does not have prefix part and
    /// - from_item_correlations are more than 1
    /// because this function has to determine which of `from1` or `from2` `field1` is from.
    ///
    /// # Failures
    ///
    /// - `SpringError::Sql` when:
    ///   - none of `from_item_correlations` has field named `pointer.column_name`
    ///   - `pointer` has a correlation but it is not any of `from_item_correlations`.
    pub(super) fn field_name(
        pointer: &FieldPointer,
        from_item_correlations: &[AliasedCorrelationName],
    ) -> Result<FieldName> {
        if from_item_correlations.is_empty() {
            unreachable!("SQL parser must handle this case")
        } else if let Some(corr) = pointer.prefix() {
            Self::field_name_with_prefix(corr, pointer.attr(), from_item_correlations)
        } else {
            Self::field_name_without_prefix(pointer.attr(), from_item_correlations)
        }
    }

    /// # Failures
    ///
    /// - `SpringError::Sql` when:
    ///   - `prefix` does not match any of `from_item_correlations`.
    fn field_name_with_prefix(
        prefix: &str,
        attr: &str,
        from_item_correlations: &[AliasedCorrelationName],
    ) -> Result<FieldName> {
        assert!(!from_item_correlations.is_empty());

        let attr = AttributeName::new(attr.to_string());
        let pointer = FieldPointer::from(format!("{}.{}", prefix, attr).as_str());

        // SELECT T.C FROM ...;
        from_item_correlations
            .iter()
            .find_map(|from_item_corr| {
                // creates AliasedFieldName to use .matches()
                let field_name_candidate = AliasedFieldName::new(
                    FieldName::new(from_item_corr.clone(), attr.clone()),
                    None,
                );
                field_name_candidate
                    .matches(&pointer)
                    .then(|| field_name_candidate.field_name)
            })
            .ok_or_else(|| {
                SpringError::Sql(anyhow!(
                    "`{}` does not match any of FROM items: {:?}",
                    pointer,
                    from_item_correlations
                ))
            })
    }

    fn field_name_without_prefix(
        attr: &str,
        from_item_correlations: &[AliasedCorrelationName],
    ) -> Result<FieldName> {
        assert!(!from_item_correlations.is_empty());
        if from_item_correlations.len() > 1 {
            return Err(SpringError::Sql(anyhow!(
                "needs pipeline info to detect which stream has the column `{:?}`",
                attr
            )));
        }

        // SELECT C FROM T (AS a)?;
        // -> C is from T
        let from_item_correlation = from_item_correlations[0].clone();
        let attr = AttributeName::new(attr.to_string());
        Ok(FieldName::new(from_item_correlation, attr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[derive(new)]
    struct TestDatum {
        index: FieldPointer,
        from_item_correlations: Vec<AliasedCorrelationName>,
        expected_result: Result<FieldName>,
    }

    #[test]
    fn test_column_reference() {
        let test_data: Vec<TestDatum> = vec![
            TestDatum::new(
                FieldPointer::from("c"),
                vec![AliasedCorrelationName::factory_sn("t")],
                Ok(FieldName::factory("t", "c")),
            ),
            TestDatum::new(
                FieldPointer::from("t.c"),
                vec![AliasedCorrelationName::factory_sn("t")],
                Ok(FieldName::factory("t", "c")),
            ),
            TestDatum::new(
                FieldPointer::from("t1.c"),
                vec![AliasedCorrelationName::factory_sn("t2")],
                Err(SpringError::Sql(anyhow!(""))),
            ),
            TestDatum::new(
                FieldPointer::from("c"),
                vec![AliasedCorrelationName::factory_sn("t").with_alias("a")],
                Ok(FieldName::factory("t", "c").with_corr_alias("a")),
            ),
            TestDatum::new(
                FieldPointer::from("t.c"),
                vec![AliasedCorrelationName::factory_sn("t").with_alias("a")],
                Ok(FieldName::factory("t", "c").with_corr_alias("a")),
            ),
            TestDatum::new(
                FieldPointer::from("a.c"),
                vec![AliasedCorrelationName::factory_sn("t").with_alias("a")],
                Ok(FieldName::factory("t", "c").with_corr_alias("a")),
            ),
            TestDatum::new(
                FieldPointer::from("x.c"),
                vec![AliasedCorrelationName::factory_sn("t").with_alias("a")],
                Err(SpringError::Sql(anyhow!(""))),
            ),
        ];

        for test_datum in test_data {
            match SelectSyntaxAnalyzer::field_name(
                &test_datum.index,
                &test_datum.from_item_correlations,
            ) {
                Ok(field_name) => {
                    assert_eq!(field_name, test_datum.expected_result.unwrap())
                }
                Err(e) => {
                    assert!(matches!(e, SpringError::Sql(_)))
                }
            }
        }
    }
}