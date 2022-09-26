use sqlparser::ast::{Expr, OrderByExpr, Query, SelectItem, SetExpr};

use super::Rule;

pub struct Orderby {}

fn extract_expr_from_select_item(item: &SelectItem) -> Result<Expr, String> {
    match item {
        sqlparser::ast::SelectItem::UnnamedExpr(e) => Ok(e.clone()),
        sqlparser::ast::SelectItem::ExprWithAlias { alias, .. } => {
            Ok(Expr::Identifier(alias.clone()))
        }
        _ => Err("Not supported".to_owned()),
    }
}

fn get_select_items(body: Box<SetExpr>) -> Result<Vec<Expr>, String> {
    match *body {
        SetExpr::Select(select) => {
            let v = select
                .projection
                .iter()
                .map(|item| extract_expr_from_select_item(item).unwrap())
                .collect::<Vec<_>>();
            Ok(v)
        }
        SetExpr::Query(query) => get_select_items(query.body.clone()),
        _ => Err("Not supported body".to_owned()),
    }
}

fn change_orderby(q: &mut Query) -> () {
    q.order_by.clear();

    let projects = get_select_items(q.body.clone()).unwrap();
    let new_orderby = projects
        .into_iter()
        .filter(|expr| match &expr {
            Expr::Value(_) => false,
            Expr::TypedString { .. } => false,
            _ => true,
        })
        .map(|expr| OrderByExpr {
            expr,
            asc: None,
            nulls_first: None,
        })
        .collect::<Vec<_>>();
    q.order_by = new_orderby;
}

impl Rule for Orderby {
    fn apply(&mut self, q: &mut Query) -> () {
        change_orderby(q)
    }
}
