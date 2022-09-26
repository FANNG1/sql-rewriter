# sql-rewriter
rewrite sql using special rules,  such as add limit, orderby

## build
1. install rust
2. cargo build

## usage
```
run sql-rewriter -h
Simple program to rewrite sql

USAGE:
    sql-rewriter [OPTIONS]

OPTIONS:
        --dialect <DIALECT>    [default: hive] [possible values: hive, mysql, ansi]
        --enable-orderby       extract orderby from select itmes and add to the sql
    -h, --help                 Print help information
        --limit <LIMIT>        if add_limit gt a negtive value, will auto add limit xx to sql
                               [default: -1]
        --print-statement      print statement parse info
    -V, --version              Print version information
```
### add order by
```
echo "SELECT a, b, 123, myfunc(b) FROM table_1 WHERE a > b AND b < 100 ORDER BY a DESC, b" | ./target/debug/sqlparser-rewriter --enable-orderby

output:
SELECT a, b, 123, myfunc(b) AS sqlrewriter-0 FROM table_1 WHERE a > b AND b < 100 ORDER BY a, b, sqlrewriter-0
```
### add limit
```
echo "SELECT a, b, 123, myfunc(b) FROM table_1 WHERE a > b AND b < 100 ORDER BY a DESC, b" | ./target/debug/sqlparser-rewriter --limit 10

output:
SELECT a, b, 123, myfunc(b) FROM table_1 WHERE a > b AND b < 100 ORDER BY a DESC, b LIMIT 10
```

## rules
* orderby
* Alias
* limit
