use clap::Parser;
use sqlparser::ast::{Expr, OrderByExpr, SelectItem, SetExpr, Statement};
use sqlparser::dialect::HiveDialect;
use std::io;

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
    limit: isize,
}

fn get_sql() -> String {
    let mut input = String::new();
    let mut n = 1;
    while n != 0 {
        match io::stdin().read_line(&mut input) {
            Ok(s) => {
                n = s;
            }
            Err(error) => println!("error: {}", error),
        }
    }
    input.trim().to_owned()
}

fn main() {
    let dialect = HiveDialect {}; // or AnsiDialect, or your own dialect ...

    let args = Args::parse();
    let sql = get_sql();
    //println!("{}",sql);

    let mut ast = sqlparser::parser::Parser::parse_sql(&dialect, &sql).unwrap();

    for s in ast.iter_mut() {
        if args.enable_orderby {
            change_orderby(s);
        }
        if args.limit >= 0 {
            add_limit(s, args.limit as usize);
        }
        println!("{}", s.to_string());
    }
}
