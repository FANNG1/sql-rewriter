use clap::{Parser, ValueEnum};
use sqlparser::ast::{Expr, Ident, OrderByExpr, Query, SelectItem, SetExpr, Statement};
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

fn get_new_ident() -> Ident {
    Ident::new(format!("sqlrewriter-{}", 0))
}

fn add_alias_to_select_items(q: &mut Query) -> Result<(), String> {
    //let mut body = *q.body.clone();
    match *q.body {
        SetExpr::Select(ref mut select) => {
            let projection_with_alias = select
                .projection
                .iter()
                .map(|item| match item {
                    SelectItem::UnnamedExpr(expr) => match expr {
                        Expr::Identifier(_) | Expr::CompoundIdentifier(_) | Expr::Value(_) => {
                            SelectItem::UnnamedExpr(expr.clone())
                        }
                        _ => SelectItem::ExprWithAlias {
                            expr: expr.clone(),
                            alias: get_new_ident(),
                        },
                    },
                    _ => item.clone(),
                })
                .collect::<Vec<_>>();
            select.projection = projection_with_alias;
        }
        SetExpr::Query(ref mut query) => {
            //let mut q = query.clone();
            return add_alias_to_select_items(&mut *query);
        }
        _ => {
            return Err("Not supported body".to_owned());
        }
    };

    Ok(())
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

fn add_limit(q: &mut Query, limit: usize) {
    q.limit = Some(Expr::Value(sqlparser::ast::Value::Number(
        format!("{}", limit),
        false,
    )))
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
    let s = ast[0].clone();

    if args.print_statement {
        println!("{:#?}", s);
        return;
    }

    let mut query = match s {
        Statement::Query(query) => *query,
        _ => {
            println!("only support query");
            return;
        }
    };

    if args.enable_orderby {
        add_alias_to_select_items(&mut query).unwrap();
        change_orderby(&mut query);
    }
    if args.limit >= 0 {
        add_limit(&mut query, args.limit as usize);
    }

    println!("{}", query.to_string());
    //println!("{}", s.to_string());
}
