use clap::{Parser, ValueEnum};
use sqlparser::ast::Statement;
use sqlparser::dialect::{AnsiDialect, Dialect, HiveDialect, MySqlDialect};
use std::io;

use crate::rules::{Alias, Limit, Orderby, Rule};
pub mod rules;

/// Simple program to rewrite sql
#[derive(clap::Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// extract orderby from select itmes and add to the sql
    #[clap(long, value_parser, default_value_t = false)]
    orderby: bool,

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

    let mut rules: Vec<Box<dyn Rule>> = vec![];

    if args.orderby {
        rules.push(Box::new(Alias::new(0)));
        rules.push(Box::new(Orderby {}))
    }
    if args.limit >= 0 {
        rules.push(Box::new(Limit::new(args.limit as usize)));
    }

    rules
        .iter_mut()
        .map(|rule| rule.apply(&mut query))
        .for_each(drop);

    println!("{}", query.to_string());
}
