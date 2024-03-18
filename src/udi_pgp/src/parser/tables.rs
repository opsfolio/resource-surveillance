use std::collections::HashSet;

use sqlparser::ast::{Expr, Query, SelectItem, SetExpr, TableFactor};
use tracing::instrument;

#[instrument(ret, level = "debug", fields(query))]
pub fn get_table_names_from_query(query: &Query) -> Vec<String> {
    let mut table_names: HashSet<String> = HashSet::new();

    //keep track of virtual tables in queries and ommitting them
    // for example `with (select * from system_info) as system_info_data select count(*) from system_info_data`
    // Should return with `system_info` (the actual table) and exclude `system_info_data` (the internal CTE name).
    let mut cte_names: HashSet<String> = HashSet::new();

    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            // CTE names are lower-cased to make comparisons easier.
            cte_names.insert(cte.alias.to_string().to_lowercase());
            table_names.extend(get_table_names_from_query(&cte.query));
        }
    }

    let from_body = get_table_names_from_set_expression(&query.body);
    table_names.extend(from_body);

    let mut output: Vec<String> = Vec::new();

    let difference = table_names.difference(&cte_names);
    for name in difference {
        // Double-check lower-case version of table name, skip any that match.
        if cte_names.contains(&name.to_lowercase()) {
            continue;
        };

        output.push(name.to_string());
    }

    output.sort();
    output
}

fn get_table_names_from_set_expression(expression: &SetExpr) -> Vec<String> {
    match expression {
        SetExpr::Select(select) => {
            let mut table_names: Vec<String> = Vec::new();

            select
                .projection
                .clone()
                .into_iter()
                .for_each(|item| match item {
                    SelectItem::UnnamedExpr(expr) => {
                        table_names.extend(get_table_names_from_expression(expr));
                    }
                    SelectItem::ExprWithAlias { expr, .. } => {
                        table_names.extend(get_table_names_from_expression(expr));
                    }
                    _ => {}
                });

            select.from.clone().into_iter().for_each(|from| {
                let from_name = get_table_names_from_table_factor(from.relation);
                table_names.extend(from_name);

                for join in from.joins {
                    let join_name = get_table_names_from_table_factor(join.relation);
                    table_names.extend(join_name);
                }
            });

            if let Some(e) = select.selection.clone() {
                table_names.extend(get_table_names_from_expression(e));
            }

            if let Some(e) = select.having.clone() {
                table_names.extend(get_table_names_from_expression(e));
            }

            let group_by = match select.group_by.clone() {
                sqlparser::ast::GroupByExpr::Expressions(exprs) => exprs,
                _ => vec![],
            };

            for exprs in [
                group_by,
                select.cluster_by.clone(),
                select.distribute_by.clone(),
                select.sort_by.clone(),
            ] {
                table_names.extend(get_table_names_from_multiple_expressions(exprs));
            }

            table_names
        }
        SetExpr::SetOperation {
            op: _value,
            set_quantifier: _pg_namespace,
            left,
            right,
        } => {
            let mut table_names = get_table_names_from_set_expression(left);
            table_names.extend(get_table_names_from_set_expression(right));
            table_names
        }
        SetExpr::Query(query) => get_table_names_from_query(query),

        SetExpr::Values(_) | SetExpr::Update(_) | SetExpr::Insert(_) | SetExpr::Table(_) => vec![],
    }
}

fn get_table_names_from_table_factor(f: TableFactor) -> Vec<String> {
    match f {
        TableFactor::Table { name, args, .. } => {
            let name = name.0.first().unwrap().value.clone();
            if args.is_some() && !args.unwrap().is_empty() && name.eq_ignore_ascii_case("unnest") {
                return Vec::new();
            }
            vec![name]
        }
        TableFactor::Derived { subquery, .. } => get_table_names_from_query(&subquery),

        TableFactor::TableFunction { expr, .. } => get_table_names_from_expression(expr),

        TableFactor::NestedJoin {
            table_with_joins,
            alias: _value,
        } => {
            let mut result = vec![];
            result.extend(get_table_names_from_table_factor(table_with_joins.relation));
            for join in table_with_joins.joins {
                result.extend(get_table_names_from_table_factor(join.relation));
            }
            result
        }
        _ => vec![],
    }
}

fn get_table_names_from_multiple_expressions(expressions: Vec<Expr>) -> Vec<String> {
    expressions
        .into_iter()
        .flat_map(get_table_names_from_expression)
        .collect::<Vec<_>>()
}

fn get_table_names_from_expression(expression: Expr) -> Vec<String> {
    match expression {
        Expr::IsNotNull(inner_expr) => get_table_names_from_expression(*inner_expr),
        Expr::IsNull(inner_expr) => get_table_names_from_expression(*inner_expr),
        Expr::InList {
            expr,
            list,
            negated: _,
        } => {
            let mut res = get_table_names_from_multiple_expressions(list);
            res.extend(get_table_names_from_expression(*expr));
            res
        }
        Expr::InSubquery {
            expr,
            subquery,
            negated: _,
        } => {
            let mut res = get_table_names_from_query(&subquery);
            res.extend(get_table_names_from_expression(*expr));
            res
        }
        _ => {
            vec![]
        }
    }
}
