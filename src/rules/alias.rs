use sqlparser::ast::{Expr, Ident, Query, SelectItem, SetExpr};

use super::Rule;

#[derive(Default)]
pub struct Alias {
  id: usize,
}

impl Alias {

    pub fn new(id: usize) ->Self {
      Self { id }
    }
    fn get_uniq_id(&mut self) -> usize {
      let id = self.id;
      self.id += 1;
      id
    }

    fn get_new_ident(&mut self) -> Ident {
        Ident::new(format!("sqlrewriter-{}", self.get_uniq_id()))
    }

    fn add_alias_to_select_items(&mut self, q: &mut Query) -> Result<(), String> {
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
                                alias: self.get_new_ident(),
                            },
                        },
                        _ => item.clone(),
                    })
                    .collect::<Vec<_>>();
                select.projection = projection_with_alias;
            }
            SetExpr::Query(ref mut query) => {
                return self.add_alias_to_select_items(&mut *query);
            }
            _ => {
                return Err("Not supported body".to_owned());
            }
        };

        Ok(())
    }
}

impl Rule for Alias {
    fn apply(&mut self, q: &mut Query) -> () {
        self.add_alias_to_select_items(q).unwrap()
    }
}
