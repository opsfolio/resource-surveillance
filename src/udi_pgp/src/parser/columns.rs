use sqlparser::ast::{Expr, Query, SelectItem, SetExpr};
use tracing::instrument;

use crate::parser::stmt::{ColumnMetadata, ExpressionType};

#[instrument(ret, level = "debug", fields(query))]
pub fn get_column_names_from_query(query: Query) -> Vec<ColumnMetadata> {
    get_column_names_from_set_expression(*query.body)
}

fn get_column_names_from_set_expression(set_expr: SetExpr) -> Vec<ColumnMetadata> {
    match set_expr {
        SetExpr::Select(select) => get_column_names_from_projection(select.projection),
        SetExpr::Query(query) => get_column_names_from_query(*query),
        SetExpr::SetOperation { left, .. } => get_column_names_from_set_expression(*left),
        _ => vec![],
    }
}

fn get_column_names_from_projection(projection: Vec<SelectItem>) -> Vec<ColumnMetadata> {
    projection
        .into_iter()
        .filter_map(|item| match item {
            SelectItem::UnnamedExpr(expr) => Some(get_column_name_from_expression(expr)),
            SelectItem::ExprWithAlias { alias, expr } => {
                let mut col = get_column_name_from_expression(expr);
                if col.name.is_empty() {
                    col.name = alias.value;
                } else {
                    col.alias = Some(alias.value);
                }
                Some(col)
            }
            SelectItem::QualifiedWildcard(qualified_wildcard, ..) => qualified_wildcard
                .0
                .first()
                .map(|identifier| ColumnMetadata {
                    name: identifier.value.clone(),
                    expr_type: ExpressionType::Wildcard,
                    alias: None,
                }),
            SelectItem::Wildcard(_) => Some(ColumnMetadata {
                name: "*".to_owned(),
                expr_type: ExpressionType::Wildcard,
                alias: None,
            }),
        })
        .collect()
}

fn get_column_name_from_expression(expr: Expr) -> ColumnMetadata {
    match expr {
        Expr::Identifier(ident) => ColumnMetadata {
            name: ident.value,
            expr_type: ExpressionType::Standard,
            alias: None,
        },
        Expr::CompoundIdentifier(compound) => {
            if let Some(last) = compound.last() {
                ColumnMetadata {
                    name: last.value.clone(),
                    expr_type: ExpressionType::Compound,
                    alias: None,
                }
            } else {
                ColumnMetadata::default() // Handle empty compound identifier
            }
        }
        Expr::Nested(e) => get_column_name_from_expression(*e),
        Expr::Function(func) => ColumnMetadata {
            name: func
                .name
                .0
                .first()
                .map_or_else(String::new, |ident| ident.value.clone()),
            expr_type: ExpressionType::Function,
            alias: None,
        },
        Expr::Case { operand, .. } => operand.map_or_else(ColumnMetadata::default, |op| {
            get_column_name_from_expression(*op)
        }),
        Expr::BinaryOp { .. } => ColumnMetadata {
            name: String::new(),
            expr_type: ExpressionType::Binary,
            alias: None,
        },
        _ => ColumnMetadata::default(), // Default case for unhandled expressions
    }
}
