use sqlparser::ast::{Expr, Query};

use super::Rule;

pub struct Limit {
    limit: usize,
}

impl Limit {
    pub fn new(limit: usize) -> Self {
        Self { limit }
    }
}

impl Rule for Limit {
    fn apply(&mut self, q: &mut Query) -> () {
        q.limit = Some(Expr::Value(sqlparser::ast::Value::Number(
            format!("{}", self.limit),
            false,
        )));
        ()
    }
}
