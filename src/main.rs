use clap::{Parser, ValueEnum};
use sqlparser::ast::{Expr, OrderByExpr, SelectItem, SetExpr, Statement};
use sqlparser::dialect::{AnsiDialect, Dialect, HiveDialect, MySqlDialect};
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

    // use sepcial dialect
    #[clap(long, arg_enum, value_parser, default_value_t = Dialect2::Hive)]
    dialect: Dialect2,

    /// print statement parse info
    #[clap(long, value_parser, default_value_t = false)]
    print_statement: bool,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
enum Dialect2 {
    Hive,
    Mysql,
    Ansi,
}

fn get_sql() -> String {
    let mut input = String::new();
    let mut n = 1;
    while n != 0 {
        n = io::stdin().read_line(&mut input).unwrap();
    }
    input.trim().to_owned()
}

fn main() {
    let args = Args::parse();

    let dialect: Box<dyn Dialect> = match &args.dialect {
        Dialect2::Mysql => Box::new(MySqlDialect {}),
        Dialect2::Hive => Box::new(HiveDialect {}),
        Dialect2::Ansi => Box::new(AnsiDialect {}),
    };

    let sql = get_sql();

    let ast = sqlparser::parser::Parser::parse_sql(&*dialect, &sql).unwrap();
    assert_eq!(ast.len(), 1);
    let mut s = ast[0].clone();

    if args.print_statement {
        println!("{:#?}", s);
        return;
    }

    if args.enable_orderby {
        change_orderby(&mut s);
    }
    if args.limit >= 0 {
        add_limit(&mut s, args.limit as usize);
    }
    println!("{}", s.to_string());
}
