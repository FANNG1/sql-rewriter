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

## rules
* orderby
* Alias
* limit
