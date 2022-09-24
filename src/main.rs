use sqlparser::ast::{Expr, OrderByExpr, SelectItem, SetExpr, Statement};
use sqlparser::dialect::HiveDialect;
//use sqlparser::parser::Parser;
use clap::Parser;

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

fn change_orderby(statement: &mut Statement) -> () {
    if let Statement::Query(q) = statement {
        q.order_by.clear();

        let projects = get_select_items(q.body.clone()).unwrap();
        println!("{:?}", projects);
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
}

fn add_limit(statement: &mut Statement, limit: usize) {
    if let Statement::Query(q) = statement {
        q.limit = Some(Expr::Value(sqlparser::ast::Value::Number(
            format!("{}", limit),
            false,
        )))
    }
}

/// Simple program to rewrite sql
#[derive(clap::Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// extract orderby from select itmes and add to the sql
    #[clap(long, value_parser, default_value_t = false)]
    enable_orderby: bool,

    /// if add_limit gt a negtive value, will auto add limit xx to sql
    #[clap(long, value_parser, default_value_t = -1)]
    add_limit: isize,
}

fn main() {
    let sql = "SELECT a, b, 123, myfunc(b) \
             FROM table_1 \
             WHERE a > b AND b < 100 \
             ORDER BY a DESC, b";

    let dialect = HiveDialect {}; // or AnsiDialect, or your own dialect ...

    let args = Args::parse();

    let mut ast = sqlparser::parser::Parser::parse_sql(&dialect, sql).unwrap();

    for s in ast.iter_mut() {
        if args.enable_orderby {
            change_orderby(s);
        }
        if args.add_limit >= 0 {
            add_limit(s, 1000);
        }
        println!("{}", s.to_string());
    }
}
