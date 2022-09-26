use sqlparser::ast::Query;
pub trait Rule {
    fn apply(&mut self, q: &mut Query) -> ();
}
